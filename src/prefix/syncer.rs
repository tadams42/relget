use std::collections::HashSet;
use std::path::Path;

use anyhow::Result;

use super::{helpers, installer, uninstaller};
use crate::{AppEntry, all_app_entries};

pub(super) fn sync(
    prefix_path: &Path, apps: &[String], configured_set: Option<&str>, offline: bool,
) -> Result<()> {
    let selected = helpers::select_apps(apps, configured_set)?;
    let entries = all_app_entries();
    let bin_dir = prefix_path.join("bin");

    let owned: HashSet<String> = entries
        .iter()
        .filter(|e| bin_dir.join(&e.exe_name).exists())
        .map(|e| e.exe_name.clone())
        .collect();
    let installed_binaries: HashSet<&str> = owned.iter().map(String::as_str).collect();

    let (to_install, to_uninstall) = compute_sync_changes(&selected, entries, &installed_binaries);

    if !to_install.is_empty() {
        log::info!("count={} prefix={:?} msg=Installing", to_install.len(), prefix_path);
        let installed = installer::install_apps(prefix_path, &to_install, offline)?;
        if !installed.is_empty() {
            println!("Installed files:");
            for path in installed {
                println!("- {}", path.display());
            }
        }
    }

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

    Ok(())
}

/// Compute the install/uninstall sets needed to reconcile the prefix with `selected`.
/// Returns `(to_install, to_uninstall)`.
pub(super) fn compute_sync_changes(
    selected: &[String], entries: &[AppEntry], installed_binaries: &HashSet<&str>,
) -> (Vec<String>, Vec<String>) {
    let selected_set: HashSet<&str> = selected.iter().map(String::as_str).collect();

    let to_install: Vec<String> = selected
        .iter()
        .filter(|id| {
            entries
                .iter()
                .find(|e| &e.id == *id)
                .is_some_and(|e| !installed_binaries.contains(e.exe_name.as_str()))
        })
        .cloned()
        .collect();

    let to_uninstall: Vec<String> = entries
        .iter()
        .filter(|e| {
            !selected_set.contains(e.id.as_str())
                && installed_binaries.contains(e.exe_name.as_str())
        })
        .map(|e| e.id.clone())
        .collect();

    (to_install, to_uninstall)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ManPagesStatus, ShellCompletionsStatus};

    fn make_entry(id: &str, exe_name: &str) -> AppEntry {
        AppEntry {
            id:                id.to_string(),
            exe_name:          exe_name.to_string(),
            url:               String::new(),
            category:          String::new(),
            description:       String::new(),
            has_musl:          false,
            man_pages:         ManPagesStatus::Unavailable,
            shell_completions: ShellCompletionsStatus::Unavailable,
        }
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
}
