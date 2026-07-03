use std::collections::HashSet;
use std::path::Path;

use anyhow::{Result, anyhow, bail};

use super::{helpers, installer, uninstaller};
use crate::{AppEntry, Registry};

pub(super) fn sync(
    prefix_path: &Path, apps: &[String], configured_set: Option<&str>, offline: bool,
) -> Result<()> {
    let selected = helpers::select_apps(apps, configured_set)?;
    let entries = Registry::entries();

    if let Some((a, b)) = helpers::find_selection_conflict(&selected, entries) {
        return Err(anyhow!(
            "cannot sync '{a}' and '{b}' together: they conflict (they provide the same binary \
             names); choose one"
        ));
    }

    let owned = helpers::bin_names_on_disk(prefix_path);
    let installed_binaries: HashSet<&str> = owned.iter().map(String::as_str).collect();

    let (to_install, to_uninstall) = compute_sync_changes(&selected, entries, &installed_binaries);

    // Uninstall first: a selected app may replace a conflicting occupant that owns the
    // same binary names, so the occupant must be gone before the replacement lands.
    if !to_uninstall.is_empty() {
        log::info!(
            "count={} prefix={:?} msg=Uninstalling",
            to_uninstall.len(),
            prefix_path
        );
        let removed = uninstaller::uninstall_apps(prefix_path, &to_uninstall)?;
        if removed.is_empty() {
            println!("No files removed.");
        } else {
            println!("Removed files:");
            for path in removed {
                println!("- {}", path.display());
            }
        }
    }

    if !to_install.is_empty() {
        log::info!("count={} prefix={:?} msg=Installing", to_install.len(), prefix_path);
        let (installed, failed) = installer::install_apps(prefix_path, &to_install, offline)?;
        if !installed.is_empty() {
            println!("Installed files:");
            for path in installed {
                println!("- {}", path.display());
            }
        }
        if failed > 0 {
            bail!("{failed} app(s) failed to install");
        }
    }

    Ok(())
}

/// Compute the install/uninstall sets needed to reconcile the prefix with `selected`.
/// Returns `(to_install, to_uninstall)`.
fn compute_sync_changes(
    selected: &[String], entries: &[AppEntry], installed_binaries: &HashSet<&str>,
) -> (Vec<String>, Vec<String>) {
    let selected_set: HashSet<&str> = selected.iter().map(String::as_str).collect();

    let to_install: Vec<String> = selected
        .iter()
        .filter(|id| {
            entries.iter().find(|e| &e.id == *id).is_some_and(|e| {
                helpers::occupying_app_id(e, entries, installed_binaries) != Some(id.as_str())
            })
        })
        .cloned()
        .collect();

    let to_uninstall: Vec<String> = entries
        .iter()
        .filter(|e| {
            !selected_set.contains(e.id.as_str())
                && helpers::occupying_app_id(e, entries, installed_binaries) == Some(e.id.as_str())
        })
        .map(|e| e.id.clone())
        .collect();

    (to_install, to_uninstall)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::compute_sync_changes;
    use crate::{AppAssetDef, AppBinaryDef, AppEntry, AssetType};

    fn make_group_entry(id: &str, binaries: &[&str], conflicts: &[&str]) -> AppEntry {
        AppEntry {
            id:                     id.to_string(),
            category_id:            String::new(),
            description:            None,
            url:                    String::new(),
            binaries:               binaries
                .iter()
                .enumerate()
                .map(|(i, name)| {
                    AppBinaryDef {
                        id:              i as u32 + 1,
                        name:            name.to_string(),
                        version_cmdline: String::new(),
                        is_main:         i == 0,
                    }
                })
                .collect(),
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
            conflicts:              conflicts.iter().map(|c| c.to_string()).collect(),
            released_version_parse: None,
        }
    }

    fn make_entry(id: &str, exe_name: &str) -> AppEntry { make_group_entry(id, &[exe_name], &[]) }

    /// qsv-style group: "qsv" and "qsv-all" share the main binary "qsv".
    fn qsv_group() -> Vec<AppEntry> {
        vec![
            make_group_entry("qsv", &["qsv"], &["qsv-all"]),
            make_group_entry("qsv-all", &["qsv", "qsvdp", "qsvlite"], &["qsv"]),
        ]
    }

    #[test]
    fn installs_selected_app_not_yet_present() {
        let entries = vec![make_entry("rg", "rg"), make_entry("bat", "bat")];
        let selected = vec!["rg".to_string(), "bat".to_string()];
        let installed = HashSet::from(["rg"]);
        let (to_install, to_uninstall) = compute_sync_changes(&selected, &entries, &installed);
        assert_eq!(to_install, ["bat"]);
        assert!(to_uninstall.is_empty());
    }

    #[test]
    fn uninstalls_installed_app_not_in_selected_set() {
        let entries = vec![make_entry("rg", "rg"), make_entry("bat", "bat")];
        let selected = vec!["rg".to_string()];
        let installed = HashSet::from(["rg", "bat"]);
        let (to_install, to_uninstall) = compute_sync_changes(&selected, &entries, &installed);
        assert!(to_install.is_empty());
        assert_eq!(to_uninstall, ["bat"]);
    }

    #[test]
    fn noop_when_installed_set_matches_selected() {
        let entries = vec![make_entry("rg", "rg"), make_entry("bat", "bat")];
        let selected = vec!["rg".to_string(), "bat".to_string()];
        let installed = HashSet::from(["rg", "bat"]);
        let (to_install, to_uninstall) = compute_sync_changes(&selected, &entries, &installed);
        assert!(to_install.is_empty());
        assert!(to_uninstall.is_empty());
    }

    #[test]
    fn handles_disjoint_installed_and_selected_sets() {
        let entries = vec![make_entry("rg", "rg"), make_entry("bat", "bat")];
        let selected = vec!["rg".to_string()];
        let installed = HashSet::from(["bat"]);
        let (to_install, to_uninstall) = compute_sync_changes(&selected, &entries, &installed);
        assert_eq!(to_install, ["rg"]);
        assert_eq!(to_uninstall, ["bat"]);
    }

    #[test]
    fn empty_installed_installs_all_selected() {
        let entries = vec![make_entry("rg", "rg"), make_entry("bat", "bat")];
        let selected = vec!["rg".to_string(), "bat".to_string()];
        let installed: HashSet<&str> = HashSet::new();
        let (mut to_install, to_uninstall) = compute_sync_changes(&selected, &entries, &installed);
        to_install.sort();
        assert_eq!(to_install, ["bat", "rg"]);
        assert!(to_uninstall.is_empty());
    }

    #[test]
    fn replaces_conflicting_occupant_with_selected_app() {
        // qsv-all occupies the prefix; syncing to [qsv] must swap them
        let entries = qsv_group();
        let selected = vec!["qsv".to_string()];
        let installed = HashSet::from(["qsv", "qsvdp", "qsvlite"]);
        let (to_install, to_uninstall) = compute_sync_changes(&selected, &entries, &installed);
        assert_eq!(to_install, ["qsv"]);
        assert_eq!(to_uninstall, ["qsv-all"]);
    }

    #[test]
    fn noop_when_selected_conflicting_app_occupies_prefix() {
        let entries = qsv_group();
        let selected = vec!["qsv-all".to_string()];
        let installed = HashSet::from(["qsv", "qsvdp", "qsvlite"]);
        let (to_install, to_uninstall) = compute_sync_changes(&selected, &entries, &installed);
        assert!(to_install.is_empty());
        assert!(to_uninstall.is_empty());
    }

    #[test]
    fn installs_conflicting_app_into_empty_prefix() {
        let entries = qsv_group();
        let selected = vec!["qsv-all".to_string()];
        let installed: HashSet<&str> = HashSet::new();
        let (to_install, to_uninstall) = compute_sync_changes(&selected, &entries, &installed);
        assert_eq!(to_install, ["qsv-all"]);
        assert!(to_uninstall.is_empty());
    }
}
