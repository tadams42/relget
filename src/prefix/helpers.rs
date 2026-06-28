use anyhow::{Result, anyhow};

use crate::{Config, Registry};

pub(super) fn get_github_token() -> Result<Option<String>> {
    let token = Config::github_token()?;
    match &token {
        Some(_) => log::info!("msg=github-token-loaded"),
        None => log::warn!("msg=github token not found; app may hit API rate limits"),
    }
    Ok(token)
}

pub(super) fn get_codeberg_token() -> Result<Option<String>> {
    let token = Config::codeberg_token()?;
    match &token {
        Some(_) => log::info!("msg=codeberg-token-loaded"),
        None => log::warn!("msg=codeberg token not found; app may hit API rate limits"),
    }
    Ok(token)
}

pub(super) fn get_gitlab_token() -> Result<Option<String>> {
    let token = Config::gitlab_token()?;
    match &token {
        Some(_) => log::info!("msg=gitlab-token-loaded"),
        None => log::warn!("msg=gitlab token not found; app may hit API rate limits"),
    }
    Ok(token)
}

pub(super) fn select_apps(
    user_chosen: &[String], configured_set: Option<&str>,
) -> Result<Vec<String>> {
    let known = Registry::global().identifiers();

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
