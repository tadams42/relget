use std::collections::{HashMap, HashSet};

use anyhow::Result;

use crate::apps::{all_app_entries, all_apps_identifiers};
use crate::installer::install_apps;
use crate::uninstaller::uninstall_apps;

use super::helpers::{get_codeberg_token, get_github_token, get_gitlab_token, select_apps};
use super::main_command::{InstallArgs, UninstallArgs, UpdateArgs};

pub fn list_apps_ids_command() {
    for id in all_apps_identifiers() {
        println!("{}", id);
    }
}

pub fn install_apps_command(args: &InstallArgs, offline: bool) -> Result<()> {
    let selected = select_apps(&args.apps, args.minimal_set, args.configured_set.as_deref())?;
    log::info!("Installing into: {:?}", args.prefix);
    let (gh_token, cb_token, gl_token) = if offline {
        (None, None, None)
    } else {
        (get_github_token()?, get_codeberg_token()?, get_gitlab_token()?)
    };
    let installed = install_apps(&args.prefix, &selected, gh_token, cb_token, gl_token, offline)?;
    if !installed.is_empty() {
        println!("Installed files:");
        for path in installed {
            println!("- {}", path.display());
        }
    }

    Ok(())
}

pub fn uninstall_command(args: &UninstallArgs) -> Result<()> {
    let selected = select_apps(&args.apps, args.minimal_set, args.configured_set.as_deref())?;
    let validated = select_apps(&selected, false, None)?;

    let removed = uninstall_apps(&args.prefix, &validated)?;
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

pub fn update_command(args: &UpdateArgs, offline: bool) -> Result<()> {
    let to_update: Vec<String> = if args.apps.is_empty()
        && !args.minimal_set
        && args.configured_set.is_none()
    {
        let bin_dir = args.prefix.join("bin");
        let installed_binaries: HashSet<String> = std::fs::read_dir(&bin_dir)
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

        if installed_binaries.is_empty() {
            println!("No binaries found in {}.", bin_dir.display());
            return Ok(());
        }

        // Build exe_name → first-match app id map; warn on collisions only when the exe is
        // installed.
        let mut exe_to_id: HashMap<&str, &str> = HashMap::new();
        for entry in all_app_entries() {
            if exe_to_id.contains_key(entry.exe_name.as_str()) {
                if installed_binaries.contains(entry.exe_name.as_str()) {
                    let winner = exe_to_id[entry.exe_name.as_str()];
                    log::warn!(
                        "exe_name '{}' maps to both '{}' and '{}'; '{}' will be used for update (re-run with --apps {} to update the other)",
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

        let ids: Vec<String> = exe_to_id
            .iter()
            .filter(|(exe, _)| installed_binaries.contains(**exe))
            .map(|(_, id)| id.to_string())
            .collect();

        if ids.is_empty() {
            println!("No relget-managed apps found in {}.", bin_dir.display());
            return Ok(());
        }

        ids
    } else {
        select_apps(&args.apps, args.minimal_set, args.configured_set.as_deref())?
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
