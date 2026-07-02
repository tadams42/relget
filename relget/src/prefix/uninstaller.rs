use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::Result;

use super::helpers;
use crate::{App, Registry};

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
    let entries = Registry::global().entries();
    let owned = helpers::bin_names_on_disk(prefix_path);
    let on_disk: HashSet<&str> = owned.iter().map(String::as_str).collect();

    let mut removed = Vec::new();
    for app_id in selected {
        // A conflicting app shares binary names with its group; removing one that is not
        // the actual occupant would cripple the app that is installed.
        let occupied_by_other = entries.iter().find(|e| &e.id == app_id).is_some_and(|e| {
            !e.conflicts.is_empty()
                && helpers::occupying_app_id(e, entries, &on_disk)
                    .is_some_and(|occupant| occupant != app_id)
        });
        if occupied_by_other {
            log::warn!(
                "app={} msg=not installed (a conflicting app occupies the prefix), skipping",
                app_id
            );
            continue;
        }

        let app = App::from_id(app_id, None, None, None, false)
            .ok_or_else(|| anyhow::anyhow!("Unknown app '{}'", app_id))?;
        removed.extend(app.uninstall(prefix_path));
    }
    Ok(removed)
}
