use anyhow::{Result, anyhow};

use crate::apps::{all_apps_identifiers, minimal_set_identifiers};
use crate::config::{load_codeberg_token, load_github_token, load_gitlab_token};

pub fn load_or_prompt_github_token(source: &str) -> Result<Option<String>> {
    match source {
        "prompt" => {
            let token = rpassword::prompt_password("GitHub API token (leave empty to skip): ")
                .unwrap_or_default();
            Ok(if token.is_empty() { None } else { Some(token) })
        }
        "load" => load_github_token(),
        _ => Err(anyhow!("Unknown token source '{}'", source)),
    }
}

pub fn load_or_prompt_codeberg_token(source: &str) -> Result<Option<String>> {
    match source {
        "prompt" => {
            let token = rpassword::prompt_password("Codeberg API token (leave empty to skip): ")
                .unwrap_or_default();
            Ok(if token.is_empty() { None } else { Some(token) })
        }
        "load" => load_codeberg_token(),
        _ => Err(anyhow!("Unknown token source '{}'", source)),
    }
}

pub fn load_or_prompt_gitlab_token(source: &str) -> Result<Option<String>> {
    match source {
        "prompt" => {
            let token = rpassword::prompt_password("GitLab API token (leave empty to skip): ")
                .unwrap_or_default();
            Ok(if token.is_empty() { None } else { Some(token) })
        }
        "load" => load_gitlab_token(),
        _ => Err(anyhow!("Unknown token source '{}'", source)),
    }
}

pub fn select_apps(user_chosen: &[String], minimal_set: bool) -> Result<Vec<String>> {
    let known: Vec<&str> = all_apps_identifiers();

    if minimal_set {
        let mut apps = minimal_set_identifiers().to_vec();
        apps.sort_unstable();
        return Ok(apps.iter().map(|s| s.to_string()).collect());
    }

    if user_chosen.is_empty() {
        return Err(anyhow!(
            "you must specify either --apps <NAME[,NAME...]> or --minimal-set; run `relget --help` for usage"
        ));
    }

    for app in user_chosen {
        if !known.contains(&app.as_str()) {
            return Err(anyhow!("Unknown app '{}'", app));
        }
    }
    Ok(user_chosen.to_vec())
}
