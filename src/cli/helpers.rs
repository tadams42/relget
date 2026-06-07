use anyhow::{Result, anyhow};

pub(super) const DEFAULT_PREFIX: &str = "/usr/local";

use crate::apps::all_apps_identifiers;
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

pub fn select_apps(user_chosen: &[String], configured_set: Option<&str>) -> Result<Vec<String>> {
    let known: Vec<&str> = all_apps_identifiers();

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
            "you must specify one of --apps <NAME[,NAME...]> or --configured-set <SET_NAME>; run `relget install --help` for usage"
        ));
    }

    for app in user_chosen {
        if !known.contains(&app.as_str()) {
            return Err(anyhow!("Unknown app '{}'", app));
        }
    }
    Ok(user_chosen.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

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
