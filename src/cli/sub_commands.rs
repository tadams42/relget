use std::collections::{HashMap, HashSet};

use anyhow::Result;

use crate::apps::{all_app_entries, all_apps_identifiers};
use crate::installer::install_apps;
use crate::uninstaller::uninstall_apps;

use super::helpers::{
    load_or_prompt_codeberg_token, load_or_prompt_github_token, load_or_prompt_gitlab_token,
    select_apps,
};
use super::main_command::Cli;

pub fn list_apps_ids_command() {
    for id in all_apps_identifiers() {
        println!("{}", id);
    }
}

pub fn install_apps_command(cli: &Cli) -> Result<()> {
    let selected = select_apps(&cli.apps, cli.minimal_set, cli.configured_set.as_deref())?;
    log::info!("Installing into: {:?}", cli.prefix);
    let (gh_token, cb_token, gl_token) = if cli.offline {
        (None, None, None)
    } else {
        (
            load_or_prompt_github_token(&cli.gh_token_source)?,
            load_or_prompt_codeberg_token(&cli.cb_token_source)?,
            load_or_prompt_gitlab_token(&cli.gl_token_source)?,
        )
    };
    let installed = install_apps(&cli.prefix, &selected, gh_token, cb_token, gl_token, cli.offline)?;
    if !installed.is_empty() {
        println!("Installed files:");
        for path in installed {
            println!("- {}", path.display());
        }
    }

    Ok(())
}

pub fn uninstall_command(cli: &Cli) -> Result<()> {
    let selected = select_apps(&cli.apps, cli.minimal_set, cli.configured_set.as_deref())?;
    let validated = select_apps(&selected, false, None)?;

    let removed = uninstall_apps(&cli.prefix, &validated)?;
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

pub fn update_command(cli: &Cli) -> Result<()> {
    let bin_dir = cli.prefix.join("bin");
    let installed_binaries: HashSet<String> = std::fs::read_dir(&bin_dir)
        .map_err(|e| anyhow::anyhow!("cannot read {}: {}", bin_dir.display(), e))?
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let path = e.path();
            if path.is_file() { Some(e.file_name().to_string_lossy().into_owned()) } else { None }
        })
        .collect();

    if installed_binaries.is_empty() {
        println!("No binaries found in {}.", bin_dir.display());
        return Ok(());
    }

    // Build exe_name → first-match app id map; warn on collisions.
    let mut exe_to_id: HashMap<&str, &str> = HashMap::new();
    for entry in all_app_entries() {
        if exe_to_id.contains_key(entry.exe_name.as_str()) {
            let winner = exe_to_id[entry.exe_name.as_str()];
            log::warn!(
                "exe_name '{}' maps to both '{}' and '{}'; '{}' will be used for update (re-run with --apps {} to update the other)",
                entry.exe_name, winner, entry.id, winner, entry.id
            );
        } else {
            exe_to_id.insert(&entry.exe_name, &entry.id);
        }
    }

    let to_update: Vec<String> = exe_to_id
        .iter()
        .filter(|(exe, _)| installed_binaries.contains(**exe))
        .map(|(_, id)| id.to_string())
        .collect();

    if to_update.is_empty() {
        println!("No relget-managed apps found in {}.", bin_dir.display());
        return Ok(());
    }

    log::info!("Updating {} app(s) in {:?}", to_update.len(), cli.prefix);
    let (gh_token, cb_token, gl_token) = if cli.offline {
        (None, None, None)
    } else {
        (
            load_or_prompt_github_token(&cli.gh_token_source)?,
            load_or_prompt_codeberg_token(&cli.cb_token_source)?,
            load_or_prompt_gitlab_token(&cli.gl_token_source)?,
        )
    };
    let installed = install_apps(&cli.prefix, &to_update, gh_token, cb_token, gl_token, cli.offline)?;
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
