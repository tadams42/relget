use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use anyhow::Result;
use clap::Args;

use crate::apps::{AppEntry, all_app_entries};
use crate::installer::install_apps;

use super::helpers::{
    DEFAULT_PREFIX, get_codeberg_token, get_github_token, get_gitlab_token, select_apps,
};

#[derive(Args)]
pub struct UpdateArgs {
    /// Install prefix (e.g. /usr/local or ~/.local)
    #[arg(short = 'p', long, default_value = DEFAULT_PREFIX)]
    pub prefix: PathBuf,

    /// App(s) to update; comma-separated.
    #[arg(
        short = 'a',
        long = "apps",
        value_name = "NAME[,NAME...]",
        value_delimiter = ',',
        conflicts_with_all = ["configured_set"]
    )]
    pub apps: Vec<String>,

    /// Load a named app set from the [sets] table in ~/.config/relget.toml
    #[arg(long, value_name = "SET_NAME", conflicts_with_all = ["apps"])]
    pub configured_set: Option<String>,
}

/// Auto-detect path: given the set of binary names present in the prefix, return the app IDs
/// that should be updated. Entries appearing first in `entries` win on exe_name collisions;
/// a warning is logged when a collision involves an installed binary.
pub(super) fn resolve_update_targets(
    installed_binaries: &HashSet<&str>, entries: &[AppEntry],
) -> Vec<String> {
    let mut exe_to_id: HashMap<&str, &str> = HashMap::new();
    for entry in entries {
        if exe_to_id.contains_key(entry.exe_name.as_str()) {
            if installed_binaries.contains(entry.exe_name.as_str()) {
                let winner = exe_to_id[entry.exe_name.as_str()];
                log::warn!(
                    "exe_name '{}' maps to both '{}' and '{}'; '{}' will be used for update \
                     (re-run with --apps {} to update the other)",
                    entry.exe_name,
                    winner,
                    entry.id,
                    winner,
                    entry.id
                );
            }
        } else {
            exe_to_id.insert(&entry.exe_name, &entry.id);
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
                .is_some_and(|e| installed_binaries.contains(e.exe_name.as_str()));
            if !present {
                log::warn!("'{}' is not installed, skipping", id);
            }
            present
        })
        .cloned()
        .collect()
}

pub fn update_command(args: &UpdateArgs, offline: bool) -> Result<()> {
    let to_update: Vec<String> = if args.apps.is_empty() && args.configured_set.is_none() {
        let bin_dir = args.prefix.join("bin");
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
        let ids = resolve_update_targets(&installed_binaries, all_app_entries());

        if ids.is_empty() {
            println!("No relget-managed apps found in {}.", bin_dir.display());
            return Ok(());
        }

        ids
    } else {
        let selected = select_apps(&args.apps, args.configured_set.as_deref())?;
        let entries = all_app_entries();
        let bin_dir = args.prefix.join("bin");
        let owned: HashSet<String> = selected
            .iter()
            .filter_map(|id| entries.iter().find(|e| &e.id == id))
            .filter(|e| bin_dir.join(&e.exe_name).exists())
            .map(|e| e.exe_name.clone())
            .collect();
        let installed_binaries: HashSet<&str> = owned.iter().map(String::as_str).collect();
        let filtered = filter_to_installed(&selected, entries, &installed_binaries);
        if filtered.is_empty() {
            println!("No installed apps to update.");
            return Ok(());
        }
        filtered
    };

    log::info!("Updating {} app(s) in {:?}", to_update.len(), args.prefix);
    let (gh_token, cb_token, gl_token) = if offline {
        (None, None, None)
    } else {
        (get_github_token()?, get_codeberg_token()?, get_gitlab_token()?)
    };
    let installed = install_apps(&args.prefix, &to_update, gh_token, cb_token, gl_token, offline)?;
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

    // --- resolve_update_targets ---

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

    // --- filter_to_installed ---

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
