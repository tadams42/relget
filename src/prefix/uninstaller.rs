use std::path::{Path, PathBuf};

use anyhow::Result;

use super::helpers;
use crate::App;

pub(super) fn uninstall(
    prefix_path: &Path, apps: &[String], configured_set: Option<&str>,
) -> Result<()> {
    let selected = helpers::select_apps(apps, configured_set)?;
    let validated = helpers::select_apps(&selected, None)?;
    let removed = uninstall_apps(prefix_path, &validated)?;
    if removed.is_empty() {
        println!("No files removed.");
    } else {
        println!("Removed files:");
        for path in removed {
            println!("- {}", path.display());
        }
    }
    Ok(())
}

pub(super) fn uninstall_apps(prefix_path: &Path, selected: &[String]) -> Result<Vec<PathBuf>> {
    let mut removed = Vec::new();
    for app_id in selected {
        let app = App::from_id(app_id, None, None, None, false)
            .ok_or_else(|| anyhow::anyhow!("Unknown app '{}'", app_id))?;
        removed.extend(app.uninstall(prefix_path));
    }
    Ok(removed)
}
