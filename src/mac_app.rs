use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

/// Represents a macOS application bundle and its metadata.
#[derive(Debug)]
pub struct MacApp {
    /// Absolute path to the .app bundle.
    pub path: PathBuf,

    /// Display name (CFBundleDisplayName, CFBundleName, or fallback to .app name).
    pub display_name: String,

    /// Bundle identifier (CFBundleIdentifier).
    pub bundle_id: String,
}

impl MacApp {
    /// Attempts to load a `MacApp` from a path to an `.app` bundle.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        // Basic validation
        if !path.exists() || path.extension().map_or(true, |ext| ext != "app") {
            bail!("Invalid macOS application bundle path: {}", path.display());
        }

        let info_plist_path = path.join("Contents/Info.plist");
        let info_plist_file = fs::File::open(info_plist_path)?;
        let info_plist: InfoPlist = plist::from_reader(info_plist_file)?;

        let display_name = info_plist
            .display_name
            .or(info_plist.name.clone())
            .or_else(|| {
                path.file_stem()
                    .map(|s| s.to_string_lossy().trim_end_matches(".app").to_string())
            })
            .with_context(|| {
                format!(
                    "No display name found in Info.plist for app bundle at {}",
                    path.display()
                )
            })?;

        let bundle_id = info_plist.bundle_id.with_context(|| {
            format!(
                "No bundle identifier found in Info.plist for app bundle at {}",
                path.display()
            )
        })?;

        Ok(MacApp {
            path: path.to_path_buf(),
            display_name,
            bundle_id,
        })
    }
}

/// Represents only the relevant fields from an Info.plist file.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct InfoPlist {
    #[serde(rename = "CFBundleIdentifier")]
    pub bundle_id: Option<String>,

    #[serde(rename = "CFBundleDisplayName")]
    pub display_name: Option<String>,

    #[serde(rename = "CFBundleName")]
    pub name: Option<String>,
}
