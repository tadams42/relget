use std::collections::HashSet;
use std::path::Path;

use anyhow::{Result, anyhow};

use crate::{AppEntry, Config, Registry};

pub(super) fn select_apps(
    user_chosen: &[String], configured_set: Option<&str>,
) -> Result<Vec<String>> {
    let known = Registry::identifiers();

    if let Some(set_name) = configured_set {
        let apps = Config::configured_set(set_name)?;
        for app in &apps {
            if !known.contains(&app.as_str()) {
                return Err(anyhow!("Unknown app '{}' in configured set '{}'", app, set_name));
            }
        }
        return Ok(apps);
    }

    if user_chosen.is_empty() {
        return Err(anyhow!(
            "you must specify one of --apps <NAME[,NAME...]> or --configured-set <SET_NAME>; run \
             `relget install --help` for usage"
        ));
    }

    for app in user_chosen {
        if !known.contains(&app.as_str()) {
            return Err(anyhow!("Unknown app '{}'", app));
        }
    }
    Ok(user_chosen.to_vec())
}

/// Names of regular files in `<prefix>/bin`; empty set when the dir doesn't exist.
pub(super) fn bin_names_on_disk(prefix_path: &Path) -> HashSet<String> {
    let Ok(dir_entries) = std::fs::read_dir(prefix_path.join("bin")) else {
        return HashSet::new();
    };
    dir_entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect()
}

/// Which app of `entry`'s conflict group occupies the prefix, judged by binaries on disk.
///
/// Apps without conflicts are simply checked for their main binary. Within a conflict group
/// the shared binary names can't identify the installed app, so the occupant is the member
/// whose complete binary set is on disk, largest set winning (`validate()` guarantees the
/// sets are distinguishable). `None` means no group member is completely present.
pub(super) fn occupying_app_id<'a>(
    entry: &'a AppEntry, entries: &'a [AppEntry], on_disk: &HashSet<&str>,
) -> Option<&'a str> {
    if entry.conflicts.is_empty() {
        return on_disk
            .contains(entry.main_exe_name())
            .then_some(entry.id.as_str());
    }
    std::iter::once(entry)
        .chain(entries.iter().filter(|e| entry.conflicts.contains(&e.id)))
        .filter(|member| {
            member
                .binaries
                .iter()
                .all(|b| on_disk.contains(b.name.as_str()))
        })
        .max_by_key(|member| member.binaries.len())
        .map(|member| member.id.as_str())
}

/// First pair of mutually conflicting apps within one selection, if any.
pub(super) fn find_selection_conflict<'a>(
    selected: &'a [String], entries: &[AppEntry],
) -> Option<(&'a str, &'a str)> {
    for (i, a) in selected.iter().enumerate() {
        let Some(entry) = entries.iter().find(|e| &e.id == a) else {
            continue;
        };
        for b in &selected[i + 1..] {
            if entry.conflicts.contains(b) {
                return Some((a.as_str(), b.as_str()));
            }
        }
    }
    None
}

/// Install-time gate: rejects selections with internal conflicts and selections that
/// conflict with an app already occupying the prefix.
pub(super) fn check_install_conflicts(
    prefix_path: &Path, selected: &[String], entries: &[AppEntry],
) -> Result<()> {
    if let Some((a, b)) = find_selection_conflict(selected, entries) {
        return Err(anyhow!(
            "cannot install '{a}' and '{b}' together: they conflict (they provide the same \
             binary names); choose one"
        ));
    }

    let with_conflicts: Vec<&AppEntry> = selected
        .iter()
        .filter_map(|id| entries.iter().find(|e| &e.id == id))
        .filter(|e| !e.conflicts.is_empty())
        .collect();
    if with_conflicts.is_empty() {
        return Ok(());
    }

    let owned = bin_names_on_disk(prefix_path);
    let on_disk: HashSet<&str> = owned.iter().map(String::as_str).collect();
    for entry in with_conflicts {
        if let Some(occupant) = occupying_app_id(entry, entries, &on_disk)
            && occupant != entry.id
        {
            return Err(anyhow!(
                "'{occupant}' is installed and conflicts with '{}'; uninstall it first \
                 (`relget uninstall --apps {occupant}`)",
                entry.id
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{find_selection_conflict, occupying_app_id, select_apps};
    use crate::{AppAssetDef, AppBinaryDef, AppEntry, AssetType};

    fn make_entry(id: &str, binaries: &[&str], conflicts: &[&str]) -> AppEntry {
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

    /// qsv-style conflict group: "small" and "large" share the main binary "shared";
    /// "large" additionally installs "extra1" and "extra2".
    fn conflict_group() -> Vec<AppEntry> {
        vec![
            make_entry("small", &["shared"], &["large"]),
            make_entry("large", &["shared", "extra1", "extra2"], &["small"]),
            make_entry("plain", &["plain"], &[]),
        ]
    }

    #[test]
    fn occupying_without_conflicts_checks_main_binary() {
        let entries = conflict_group();
        let plain = &entries[2];
        assert_eq!(
            occupying_app_id(plain, &entries, &HashSet::from(["plain"])),
            Some("plain")
        );
        assert_eq!(occupying_app_id(plain, &entries, &HashSet::new()), None);
    }

    #[test]
    fn occupying_small_set_on_disk_picks_small() {
        let entries = conflict_group();
        let on_disk = HashSet::from(["shared"]);
        // same answer no matter which group member we ask from
        assert_eq!(occupying_app_id(&entries[0], &entries, &on_disk), Some("small"));
        assert_eq!(occupying_app_id(&entries[1], &entries, &on_disk), Some("small"));
    }

    #[test]
    fn occupying_full_set_on_disk_picks_large() {
        let entries = conflict_group();
        let on_disk = HashSet::from(["shared", "extra1", "extra2"]);
        assert_eq!(occupying_app_id(&entries[0], &entries, &on_disk), Some("large"));
        assert_eq!(occupying_app_id(&entries[1], &entries, &on_disk), Some("large"));
    }

    #[test]
    fn occupying_partial_large_set_falls_back_to_small() {
        let entries = conflict_group();
        let on_disk = HashSet::from(["shared", "extra1"]); // extra2 missing
        assert_eq!(occupying_app_id(&entries[0], &entries, &on_disk), Some("small"));
        assert_eq!(occupying_app_id(&entries[1], &entries, &on_disk), Some("small"));
    }

    #[test]
    fn occupying_empty_prefix_returns_none() {
        let entries = conflict_group();
        assert_eq!(occupying_app_id(&entries[0], &entries, &HashSet::new()), None);
        assert_eq!(occupying_app_id(&entries[1], &entries, &HashSet::new()), None);
    }

    #[test]
    fn selection_conflict_detects_conflicting_pair() {
        let entries = conflict_group();
        let selected = vec![
            "plain".to_string(),
            "small".to_string(),
            "large".to_string(),
        ];
        assert_eq!(find_selection_conflict(&selected, &entries), Some(("small", "large")));
    }

    #[test]
    fn selection_conflict_none_for_compatible_apps() {
        let entries = conflict_group();
        let selected = vec!["plain".to_string(), "large".to_string()];
        assert_eq!(find_selection_conflict(&selected, &entries), None);
    }

    #[test]
    fn select_apps_accepts_known_id() {
        let result = select_apps(&["bat".to_string()], None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ["bat"]);
    }

    #[test]
    fn select_apps_rejects_unknown_id() {
        assert!(select_apps(&["nonexistent_app_xyz".to_string()], None).is_err());
    }

    #[test]
    fn select_apps_requires_at_least_one_selector() {
        assert!(select_apps(&[], None).is_err());
    }

    #[test]
    fn select_apps_accepts_multiple_known_ids() {
        let result = select_apps(&["bat".to_string(), "ripgrep".to_string()], None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ["bat", "ripgrep"]);
    }
}
