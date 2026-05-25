pub mod apps;
pub mod archive;
pub mod cache;
pub mod codeberg;
pub mod github;
pub mod installer;
pub mod types;
pub mod uninstaller;
pub mod version;

use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};

use apps::{all_app_entries, create_app};
use uninstaller::uninstall_app;

pub const DEFAULT_PREFIX: &str = "/usr/local";

pub fn known_apps_identifiers() -> Vec<&'static str> {
    let mut ids: Vec<&'static str> = all_app_entries().iter().map(|e| e.id).collect();
    ids.sort_unstable();
    ids
}

pub fn select_apps(user_chosen: &[String], minimal_set: bool) -> Result<Vec<String>> {
    let known: Vec<&str> = known_apps_identifiers();

    if minimal_set {
        let mut apps = MINIMAL_SET.to_vec();
        apps.sort_unstable();
        return Ok(apps.iter().map(|s| s.to_string()).collect());
    }

    if user_chosen.is_empty() {
        return Ok(known.iter().map(|s| s.to_string()).collect());
    }

    for app in user_chosen {
        if !known.contains(&app.as_str()) {
            return Err(anyhow!("Unknown app '{}'", app));
        }
    }
    Ok(user_chosen.to_vec())
}

pub fn install_apps(
    prefix: &Path, selected: &[String], gh_token: Option<String>, cb_token: Option<String>,
    offline: bool,
) -> Result<Vec<PathBuf>> {
    let mut installed = Vec::new();
    for app_id in selected {
        let app = create_app(app_id, gh_token.clone(), cb_token.clone(), offline)
            .ok_or_else(|| anyhow!("Unknown app '{}'", app_id))?;
        match app.install(prefix) {
            Ok(paths) => installed.extend(paths),
            Err(e) => {
                if offline {
                    log::warn!("app={} msg=Skipping (offline, no cached data): {:#}", app_id, e);
                } else {
                    log::error!("app={} msg=Install failed: {:#}", app_id, e);
                }
            }
        }
    }
    Ok(installed)
}

pub fn uninstall_apps(prefix: &Path, selected: &[String]) -> Result<Vec<PathBuf>> {
    let validated = select_apps(selected, false)?;
    let mut removed = Vec::new();
    for app_id in &validated {
        let app = create_app(app_id, None, None, false)
            .ok_or_else(|| anyhow!("Unknown app '{}'", app_id))?;
        removed.extend(uninstall_app(prefix, app.exe_name()));
    }
    Ok(removed)
}

pub fn resolve_github_token(source: &str) -> Result<Option<String>> {
    match source {
        "prompt" => {
            let token = rpassword::prompt_password("GitHub API token (leave empty to skip): ")
                .unwrap_or_default();
            Ok(if token.is_empty() { None } else { Some(token) })
        }
        "load" => {
            if let Ok(t) = std::env::var("GITHUB_API_TOKEN") {
                if !t.is_empty() {
                    return Ok(Some(t));
                }
            }
            let config_path = dirs::home_dir()
                .unwrap_or_default()
                .join(".config")
                .join("github")
                .join("api_token");
            if config_path.exists() {
                let content = std::fs::read_to_string(&config_path)?;
                let token = content.lines().last().unwrap_or("").trim().to_string();
                return Ok(if token.is_empty() { None } else { Some(token) });
            }
            Ok(None)
        }
        _ => Err(anyhow!("Unknown token source '{}'", source)),
    }
}

pub fn resolve_codeberg_token(source: &str) -> Result<Option<String>> {
    match source {
        "prompt" => {
            let token = rpassword::prompt_password("Codeberg API token (leave empty to skip): ")
                .unwrap_or_default();
            Ok(if token.is_empty() { None } else { Some(token) })
        }
        "load" => {
            if let Ok(t) = std::env::var("CODEBERG_API_TOKEN") {
                if !t.is_empty() {
                    return Ok(Some(t));
                }
            }
            let config_path = dirs::home_dir()
                .unwrap_or_default()
                .join(".config")
                .join("codeberg")
                .join("api_token");
            if config_path.exists() {
                let content = std::fs::read_to_string(&config_path)?;
                let token = content.lines().last().unwrap_or("").trim().to_string();
                return Ok(if token.is_empty() { None } else { Some(token) });
            }
            Ok(None)
        }
        _ => Err(anyhow!("Unknown token source '{}'", source)),
    }
}

#[rustfmt::skip]
pub const MINIMAL_SET: &[&str] = &[
    apps::files::bat::Bat::ID,
    apps::chezmoi::Chezmoi::ID,
    apps::containers::d4s::D4S::ID,
    apps::git::delta::Delta::ID,
    apps::git::difftastic::Difftastic::ID,
    apps::files::dust::Dust::ID,
    apps::files::eza::Eza::ID,
    apps::files::fd_find::FdFind::ID,
    apps::dev_envs::fnm::Fnm::ID,
    apps::shell::fzf::Fzf::ID,
    apps::git::gitleaks::Gitleaks::ID,
    apps::data::gojq::GoJq::ID,
    apps::data::jq::Jq::ID,
    apps::git::lazygit::Lazygit::ID,
    apps::rclone::Rclone::ID,
    apps::http::restish::Restish::ID,
    apps::files::ripgrep::Ripgrep::ID,
    apps::files::sd_edit::SdEdit::ID,
    apps::shell::starship::Starship::ID,
    apps::dev_envs::uv::Uv::ID,
    apps::data::yq::Yq::ID,
    apps::shell::zoxide::Zoxide::ID,
];
