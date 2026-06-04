use anyhow::{Result, anyhow};

use crate::apps::{all_apps_identifiers, minimal_set_identifiers};
use crate::config::{
    load_codeberg_token, load_configured_set, load_github_token, load_gitlab_token,
};

pub fn load_or_prompt_github_token(source: &str) -> Result<Option<String>> {
    let token = match source {
        "prompt" => {
            let token = rpassword::prompt_password("GitHub API token (leave empty to skip): ")
                .unwrap_or_default();
            Ok(if token.is_empty() { None } else { Some(token) })
        }
        "load" => load_github_token(),
        _ => Err(anyhow!("Unknown token source '{}'", source)),
    }?;
    match &token {
        Some(_) => log::info!("GitHub token found and loaded"),
        None => log::warn!("GitHub token not found; app may hit API rate limits"),
    }
    Ok(token)
}

pub fn load_or_prompt_codeberg_token(source: &str) -> Result<Option<String>> {
    let token = match source {
        "prompt" => {
            let token = rpassword::prompt_password("Codeberg API token (leave empty to skip): ")
                .unwrap_or_default();
            Ok(if token.is_empty() { None } else { Some(token) })
        }
        "load" => load_codeberg_token(),
        _ => Err(anyhow!("Unknown token source '{}'", source)),
    }?;
    match &token {
        Some(_) => log::info!("Codeberg token found and loaded"),
        None => log::warn!("Codeberg token not found; app may hit API rate limits"),
    }
    Ok(token)
}

pub fn load_or_prompt_gitlab_token(source: &str) -> Result<Option<String>> {
    let token = match source {
        "prompt" => {
            let token = rpassword::prompt_password("GitLab API token (leave empty to skip): ")
                .unwrap_or_default();
            Ok(if token.is_empty() { None } else { Some(token) })
        }
        "load" => load_gitlab_token(),
        _ => Err(anyhow!("Unknown token source '{}'", source)),
    }?;
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
