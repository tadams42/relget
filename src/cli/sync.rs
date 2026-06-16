use std::collections::HashSet;
use std::path::PathBuf;

use anyhow::Result;
use clap::Args;

use super::install::install_apps;
use super::uninstall::uninstall_apps;
use crate::apps::{AppEntry, all_app_entries};

use super::helpers::{
    DEFAULT_PREFIX, get_codeberg_token, get_github_token, get_gitlab_token, select_apps,
};

#[derive(Args)]
pub struct SyncArgs {
    /// Install prefix (e.g. /usr/local or ~/.local)
    #[arg(short = 'p', long, default_value = DEFAULT_PREFIX)]
    pub prefix: PathBuf,

    /// App(s) to sync; comma-separated.
    #[arg(
        short = 'a',
        long = "apps",
        value_name = "NAME[,NAME...]",
        value_delimiter = ',',
        conflicts_with_all = ["configured_set"]
    )]
    pub apps: Vec<String>,

    #[arg(
        long,
        value_name = "SET_NAME",
        conflicts_with_all = ["apps"],
        long_help = "Load a named app set from the [sets] table in ~/.config/relget.toml"
    )]
    pub configured_set: Option<String>,
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

pub fn sync_command(args: &SyncArgs, offline: bool) -> Result<()> {
    let selected = select_apps(&args.apps, args.configured_set.as_deref())?;
    let entries = all_app_entries();
    let bin_dir = args.prefix.join("bin");

    let owned: HashSet<String> = entries
        .iter()
        .filter(|e| bin_dir.join(&e.exe_name).exists())
        .map(|e| e.exe_name.clone())
        .collect();
    let installed_binaries: HashSet<&str> = owned.iter().map(String::as_str).collect();

    let (to_install, to_uninstall) = compute_sync_changes(&selected, entries, &installed_binaries);

    if !to_install.is_empty() {
        log::info!("count={} prefix={:?} msg=Installing", to_install.len(), args.prefix);
        let (gh_token, cb_token, gl_token) = if offline {
            (None, None, None)
        } else {
            (get_github_token()?, get_codeberg_token()?, get_gitlab_token()?)
        };
        let installed =
            install_apps(&args.prefix, &to_install, gh_token, cb_token, gl_token, offline)?;
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
            args.prefix
        );
        let removed = uninstall_apps(&args.prefix, &to_uninstall)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::apps::{ManPagesStatus, ShellCompletionsStatus};

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
