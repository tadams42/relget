use anyhow::{Result, anyhow};

use crate::apps::{all_apps_identifiers, minimal_set_identifiers};
use crate::config::{
    load_codeberg_token, load_configured_set, load_github_token, load_gitlab_token,
};

pub fn get_github_token() -> Result<Option<String>> {
    let token = load_github_token()?;
    match &token {
        Some(_) => log::info!("GitHub token found and loaded"),
        None => log::warn!("GitHub token not found; app may hit API rate limits"),
    }
    Ok(token)
}

pub fn get_codeberg_token() -> Result<Option<String>> {
    let token = load_codeberg_token()?;
    match &token {
        Some(_) => log::info!("Codeberg token found and loaded"),
        None => log::warn!("Codeberg token not found; app may hit API rate limits"),
    }
    Ok(token)
}

pub fn get_gitlab_token() -> Result<Option<String>> {
    let token = load_gitlab_token()?;
    match &token {
        Some(_) => log::info!("GitLab token found and loaded"),
        None => log::warn!("GitLab token not found; app may hit API rate limits"),
    }
    Ok(token)
}

pub fn select_apps(
    user_chosen: &[String], minimal_set: bool, configured_set: Option<&str>,
) -> Result<Vec<String>> {
    let known: Vec<&str> = all_apps_identifiers();

    if minimal_set {
        let mut apps = minimal_set_identifiers().to_vec();
        apps.sort_unstable();
        return Ok(apps.iter().map(|s| s.to_string()).collect());
    }

    if let Some(set_name) = configured_set {
        let apps = load_configured_set(set_name)?;
        for app in &apps {
            if !known.contains(&app.as_str()) {
                return Err(anyhow!("Unknown app '{}' in configured set '{}'", app, set_name));
            }
        }
        return Ok(apps);
    }

    if user_chosen.is_empty() {
        return Err(anyhow!(
            "you must specify one of --apps <NAME[,NAME...]>, --minimal-set, or --configured-set <SET_NAME>; run `relget install --help` for usage"
        ));
    }

    for app in user_chosen {
        if !known.contains(&app.as_str()) {
            return Err(anyhow!("Unknown app '{}'", app));
        }
    }
    Ok(user_chosen.to_vec())
}
