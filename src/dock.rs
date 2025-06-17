use crate::mac_app::MacApp;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Represents the top-level structure of the macOS Dock configuration plist.
#[derive(Debug, Serialize, Deserialize)]
pub struct Dock {
    /// Applications pinned to the Dock (left side).
    #[serde(rename = "persistent-apps")]
    pub applications: Option<Vec<DockItem>>,

    /// Non-application items like folders, documents, or spacers (right side).
    #[serde(rename = "persistent-others")]
    pub others: Option<Vec<DockItem>>,
}

impl Dock {
    /// Loads the Dock configuration from the user's preferences plist file.
    pub fn load() -> Result<Self> {
        let dock_path = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
            .join("Library/Preferences/com.apple.dock.plist");

        let file = std::fs::File::open(&dock_path)
            .with_context(|| format!("Failed to open Dock plist at {}", dock_path.display()))?;

        let dock: Dock = plist::from_reader(file)
            .with_context(|| format!("Failed to parse Dock plist at {}", dock_path.display()))?;

        Ok(dock)
    }

    /// Adds a new application to the Dock's persistent applications section.
    pub fn add_app(&mut self, app: &MacApp) {
        if self.applications.is_none() {
            self.applications = Some(Vec::new());
        }
        if let Some(apps) = &mut self.applications {
            apps.push(DockItem::new(app));
        }
    }

    /// Restart the Dock process to apply changes.
    pub fn restart() -> Result<()> {
        std::process::Command::new("killall")
            .arg("Dock")
            .status()
            .with_context(|| "Failed to restart the Dock")?;
        Ok(())
    }
}

/// Represents an individual item in the Dock.
#[derive(Debug, Serialize, Deserialize)]
pub struct DockItem {
    /// Metadata associated with this Dock item.
    #[serde(rename = "tile-data")]
    pub metadata: TileMetadata,

    /// The kind of item: application, folder, document, or spacer.
    #[serde(rename = "tile-type")]
    pub kind: DockItemKind,
}

impl DockItem {
    pub fn new(app: &MacApp) -> Self {
        DockItem {
            kind: DockItemKind::FileTile,
            metadata: TileMetadata {
                location: Some(FileLocation {
                    url: format!("file://{}", app.path.display()),
                    url_type: 15, // Standard file URL type
                }),
                display_name: Some(app.display_name.clone()),
                bundle_id: Some(app.bundle_id.clone()),
            },
        }
    }
}

/// Describes the type of Dock item.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DockItemKind {
    /// A file-based app or document.
    FileTile,

    /// A folder shortcut in the Dock.
    DirectoryTile,

    /// A visual spacer between items.
    SpacerTile,

    /// Unknown or future Dock tile types (fallback).
    #[serde(other)]
    Unknown,
}

/// Contains metadata for a Dock item (path, label, etc.).
#[derive(Debug, Serialize, Deserialize)]
pub struct TileMetadata {
    /// The location on disk for the Dock item.
    #[serde(rename = "file-data")]
    pub location: Option<FileLocation>,

    /// The display name shown under the icon in the Dock.
    #[serde(rename = "file-label")]
    pub display_name: Option<String>,

    /// The app's bundle identifier, if applicable.
    #[serde(rename = "bundle-identifier")]
    pub bundle_id: Option<String>,
}

/// Represents the file system URL and URL type.
#[derive(Debug, Serialize, Deserialize)]
pub struct FileLocation {
    /// The item's path as a `file://` URL string.
    #[serde(rename = "_CFURLString")]
    pub url: String,

    /// The URL type, typically 15 for file URLs.
    #[serde(rename = "_CFURLStringType")]
    pub url_type: i32,
}
