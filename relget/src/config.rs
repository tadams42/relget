use std::collections::HashMap;
use std::sync::LazyLock;

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

/// Config file parsed once per process. The error arm keeps the message as a `String`
/// because `anyhow::Error` is not `Clone`; accessors re-wrap it on each use.
static CONFIG: LazyLock<Result<RelgetConfig, String>> = LazyLock::new(|| {
    let Some(path) = config_path() else {
        return Ok(RelgetConfig::default());
    };
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    toml::from_str(&text).map_err(|e| e.to_string())
});

fn config() -> Result<&'static RelgetConfig> {
    CONFIG
        .as_ref()
        .map_err(|e| anyhow!("loading ~/.config/relget/config.toml: {e}"))
}

pub struct Config;

impl Config {
    /// Loads a forge API token from `env_var`, falling back to the config file, and logs
    /// whether one was found.
    fn token(
        provider: &str, env_var: &str, from_config: fn(&RelgetConfig) -> &Option<String>,
    ) -> Result<Option<String>> {
        let token = match std::env::var(env_var) {
            Ok(t) if !t.is_empty() => Some(t),
            _ => from_config(config()?).clone(),
        };
        match &token {
            Some(_) => log::info!("msg={provider}-token-loaded"),
            None => log::warn!("msg={provider} token not found; relget may hit API rate limits"),
        }
        Ok(token)
    }

    pub fn github_token() -> Result<Option<String>> {
        Self::token("github", "RELGET_GHB_TOKEN", |c| &c.github_token)
    }

    pub fn codeberg_token() -> Result<Option<String>> {
        Self::token("codeberg", "RELGET_CDB_TOKEN", |c| &c.codeberg_token)
    }

    pub fn gitlab_token() -> Result<Option<String>> {
        Self::token("gitlab", "RELGET_GLB_TOKEN", |c| &c.gitlab_token)
    }

    pub fn configured_set(name: &str) -> Result<Vec<String>> {
        config()?.sets.get(name).cloned().ok_or_else(|| {
            anyhow!(
                "no configured set '{}' found in ~/.config/relget/config.toml under [sets]",
                name
            )
        })
    }
}
