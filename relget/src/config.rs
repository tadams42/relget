use std::collections::HashMap;

use anyhow::{Result, anyhow};
use serde::Deserialize;

#[derive(Deserialize, Default)]
struct RelgetConfig {
    github_token:   Option<String>,
    codeberg_token: Option<String>,
    gitlab_token:   Option<String>,
    #[serde(default)]
    sets:           HashMap<String, Vec<String>>,
}

fn config_path() -> Option<std::path::PathBuf> {
    xdg::BaseDirectories::with_prefix("relget").find_config_file("config.toml")
}

fn load_config() -> Result<RelgetConfig> {
    let Some(path) = config_path() else {
        return Ok(RelgetConfig::default());
    };
    Ok(toml::from_str(&std::fs::read_to_string(&path)?)?)
}

pub struct Config;

impl Config {
    pub fn github_token() -> Result<Option<String>> {
        if let Ok(t) = std::env::var("RELGET_GHB_TOKEN") {
            if !t.is_empty() {
                return Ok(Some(t));
            }
        }
        Ok(load_config()?.github_token)
    }

    pub fn codeberg_token() -> Result<Option<String>> {
        if let Ok(t) = std::env::var("RELGET_CDB_TOKEN") {
            if !t.is_empty() {
                return Ok(Some(t));
            }
        }
        Ok(load_config()?.codeberg_token)
    }

    pub fn gitlab_token() -> Result<Option<String>> {
        if let Ok(t) = std::env::var("RELGET_GLB_TOKEN") {
            if !t.is_empty() {
                return Ok(Some(t));
            }
        }
        Ok(load_config()?.gitlab_token)
    }

    pub fn configured_set_names() -> Result<Vec<String>> {
        Ok(load_config()?.sets.into_keys().collect())
    }

    pub fn configured_set(name: &str) -> Result<Vec<String>> {
        let config = load_config()?;
        config.sets.get(name).cloned().ok_or_else(|| {
            anyhow!(
                "no configured set '{}' found in ~/.config/relget/config.toml under [sets]",
                name
            )
        })
    }
}
