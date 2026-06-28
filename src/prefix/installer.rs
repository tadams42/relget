use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};

use super::helpers;
use crate::{App, AppEntry, RateLimitError, Registry};

pub(super) fn install(
    prefix_path: &Path, apps: &[String], configured_set: Option<&str>, offline: bool,
) -> Result<()> {
    log::info!("prefix={:?} msg=Installing", prefix_path);

    let selected = helpers::select_apps(apps, configured_set)?;
    let installed = install_apps(prefix_path, &selected, offline)?;

    if !installed.is_empty() {
        println!("Installed files:");
        for path in installed {
            println!("- {}", path.display());
        }
    }

    Ok(())
}

pub(super) fn update(
    prefix_path: &Path, apps: &[String], configured_set: Option<&str>, offline: bool,
) -> Result<()> {
    let to_update: Vec<String> = if apps.is_empty() && configured_set.is_none() {
        let bin_dir = prefix_path.join("bin");
        let owned: HashSet<String> = std::fs::read_dir(&bin_dir)
            .map_err(|e| anyhow::anyhow!("cannot read {}: {}", bin_dir.display(), e))?
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let path = e.path();
                if path.is_file() {
                    Some(e.file_name().to_string_lossy().into_owned())
                } else {
                    None
                }
            })
            .collect();

        if owned.is_empty() {
            println!("No binaries found in {}.", bin_dir.display());
            return Ok(());
        }

        let installed_binaries: HashSet<&str> = owned.iter().map(String::as_str).collect();
        let ids = resolve_update_targets(&installed_binaries, Registry::global().entries());

        if ids.is_empty() {
            println!("No relget-managed apps found in {}.", bin_dir.display());
            return Ok(());
        }

        ids
    } else {
        let selected = helpers::select_apps(apps, configured_set)?;
        let entries = Registry::global().entries();
        let bin_dir = prefix_path.join("bin");
        let owned: HashSet<String> = selected
            .iter()
            .filter_map(|id| entries.iter().find(|e| &e.id == id))
            .filter(|e| bin_dir.join(e.main_exe_name()).exists())
            .map(|e| e.main_exe_name().to_owned())
            .collect();
        let installed_binaries: HashSet<&str> = owned.iter().map(String::as_str).collect();
        let filtered = filter_to_installed(&selected, entries, &installed_binaries);
        if filtered.is_empty() {
            println!("No installed apps to update.");
            return Ok(());
        }
        filtered
    };

    log::info!("count={} prefix={:?} msg=Updating", to_update.len(), prefix_path);
    let installed = install_apps(prefix_path, &to_update, offline)?;
    if installed.is_empty() {
        println!("All apps already at latest version.");
    } else {
        println!("Installed files:");
        for path in installed {
            println!("- {}", path.display());
        }
    }

    Ok(())
}

pub(super) fn install_apps(
    prefix_path: &Path, selected: &[String], offline: bool,
) -> Result<Vec<PathBuf>> {
    let (gh_token, cb_token, gl_token) = if offline {
        (None, None, None)
    } else {
        (
            helpers::get_github_token()?,
            helpers::get_codeberg_token()?,
            helpers::get_gitlab_token()?,
        )
    };
    let mut installed = Vec::new();
    for app_id in selected {
        let app =
            App::from_id(app_id, gh_token.clone(), cb_token.clone(), gl_token.clone(), offline)
                .ok_or_else(|| anyhow!("Unknown app '{}'", app_id))?;
        match app.install(prefix_path) {
            Ok(paths) => installed.extend(paths),
            Err(e) => {
                if e.chain().any(|cause| cause.is::<RateLimitError>()) {
                    log::warn!("app={} msg=Skipping (rate limit): {}", app_id, e.root_cause());
                } else if offline {
                    log::warn!("app={} msg=Skipping (offline, no cached data): {:#}", app_id, e);
                } else {
                    log::error!("app={} msg=Install failed: {:#}", app_id, e);
                }
            }
        }
    }
    Ok(installed)
}

/// Auto-detect path: given the set of binary names present in the prefix, return the app IDs
/// that should be updated. Entries appearing first in `entries` win on exe_name collisions;
/// a warning is logged when a collision involves an installed binary.
pub(super) fn resolve_update_targets(
    installed_binaries: &HashSet<&str>, entries: &[AppEntry],
) -> Vec<String> {
    let mut exe_to_id: HashMap<&str, &str> = HashMap::new();
    for entry in entries {
        let exe = entry.main_exe_name();
        if exe_to_id.contains_key(exe) {
            if installed_binaries.contains(exe) {
                let winner = exe_to_id[exe];
                log::warn!(
                    "exe_name={} winner={} duplicate={} msg=ambiguous exe_name; re-run with \
                    --apps {} to update the other",
                    exe,
                    winner,
                    entry.id,
                    entry.id
                );
            }
        } else {
            exe_to_id.insert(exe, &entry.id);
        }
    }

    exe_to_id
        .iter()
        .filter(|(exe, _)| installed_binaries.contains(**exe))
        .map(|(_, id)| id.to_string())
        .collect()
}

/// Explicit path: keep only those selected app IDs whose binary is present in the prefix.
pub(super) fn filter_to_installed(
    selected: &[String], entries: &[AppEntry], installed_binaries: &HashSet<&str>,
) -> Vec<String> {
    selected
        .iter()
        .filter(|id| {
            let present = entries
                .iter()
                .find(|e| &e.id == *id)
                .is_some_and(|e| installed_binaries.contains(e.main_exe_name()));
            if !present {
                log::warn!("app={} msg=not installed, skipping", id);
            }
            present
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AppAssetDef, AppBinaryDef, AssetType};

    fn make_entry(id: &str, exe_name: &str) -> AppEntry {
        AppEntry {
            id:                     id.to_string(),
            category_id:            String::new(),
            description:            None,
            url:                    String::new(),
            has_musl:               false,
            binaries:               vec![AppBinaryDef {
                id:              1,
                name:            exe_name.to_string(),
                version_cmdline: String::new(),
                is_main:         true,
            }],
            assets:                 vec![AppAssetDef {
                id:           1,
                asset_type:   AssetType::Archive,
                starts_with:  None,
                contains:     None,
                not_contains: None,
                ends_with:    None,
                equals:       None,
            }],
            shell_completions:      vec![],
            man_pages:              vec![],
            released_version_parse: None,
        }
    }

    #[test]
    fn resolve_matches_installed_binary() {
        let entries = vec![make_entry("rg", "rg"), make_entry("bat", "bat")];
        let installed = HashSet::from(["rg"]);
        let result = resolve_update_targets(&installed, &entries);
        assert_eq!(result, ["rg"]);
    }

    #[test]
    fn resolve_ignores_unregistered_binary() {
        let entries = vec![make_entry("rg", "rg")];
        let installed = HashSet::from(["rg", "something_untracked"]);
        let result = resolve_update_targets(&installed, &entries);
        assert_eq!(result, ["rg"]);
    }

    #[test]
    fn resolve_collision_keeps_first_entry() {
        let entries = vec![make_entry("qsv", "qsv"), make_entry("qsv_all", "qsv")];
        let installed = HashSet::from(["qsv"]);
        let result = resolve_update_targets(&installed, &entries);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "qsv");
    }

    #[test]
    fn resolve_returns_empty_when_nothing_matches() {
        let entries = vec![make_entry("rg", "rg")];
        let installed: HashSet<&str> = HashSet::new();
        let result = resolve_update_targets(&installed, &entries);
        assert!(result.is_empty());
    }

    #[test]
    fn resolve_returns_multiple_matches() {
        let entries = vec![make_entry("rg", "rg"), make_entry("bat", "bat")];
        let installed = HashSet::from(["rg", "bat"]);
        let mut result = resolve_update_targets(&installed, &entries);
        result.sort();
        assert_eq!(result, ["bat", "rg"]);
    }

    #[test]
    fn filter_keeps_apps_whose_binary_is_present() {
        let entries = vec![make_entry("rg", "rg"), make_entry("bat", "bat")];
        let installed = HashSet::from(["rg"]);
        let selected = vec!["rg".to_string(), "bat".to_string()];
        let result = filter_to_installed(&selected, &entries, &installed);
        assert_eq!(result, ["rg"]);
    }

    #[test]
    fn filter_drops_apps_whose_binary_is_absent() {
        let entries = vec![make_entry("rg", "rg")];
        let installed: HashSet<&str> = HashSet::new();
        let selected = vec!["rg".to_string()];
        let result = filter_to_installed(&selected, &entries, &installed);
        assert!(result.is_empty());
    }

    #[test]
    fn filter_returns_all_when_all_are_installed() {
        let entries = vec![make_entry("rg", "rg"), make_entry("bat", "bat")];
        let installed = HashSet::from(["rg", "bat"]);
        let selected = vec!["rg".to_string(), "bat".to_string()];
        let result = filter_to_installed(&selected, &entries, &installed);
        assert_eq!(result, ["rg", "bat"]);
    }
}
