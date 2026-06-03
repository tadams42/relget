use super::containers::D4S;
use super::data::{GoJq, Jq, Yq};
use super::dev_envs::{Fnm, Uv};
use super::files::{Bat, Dust, Dysk, Eza, FdFind, Ripgrep, SdEdit};
use super::git::{Delta, Difftastic, Gitleaks, Lazygit};
use super::http::Restish;
use super::other::Chezmoi;
use super::shell::{Fzf, Starship, Zoxide};
use rust_embed::RustEmbed;
use serde::Deserialize;
use std::sync::OnceLock;

#[derive(RustEmbed)]
#[folder = "src/apps/"]
#[include = "registry.yaml"]
struct RegistryAsset;

#[derive(Debug, Deserialize)]
pub struct AppEntry {
    pub id:          String,
    pub url:         String,
    pub category:    String,
    pub description: String,
}

#[rustfmt::skip]
pub const MINIMAL_SET: &[&str] = &[
    Bat::ID,
    Chezmoi::ID,
    D4S::ID,
    Delta::ID,
    Difftastic::ID,
    Dust::ID,
    Dysk::ID,
    Eza::ID,
    FdFind::ID,
    Fnm::ID,
    Fzf::ID,
    Gitleaks::ID,
    GoJq::ID,
    Jq::ID,
    Lazygit::ID,
    Restish::ID,
    Ripgrep::ID,
    SdEdit::ID,
    Starship::ID,
    Uv::ID,
    Yq::ID,
    Zoxide::ID,
];

static REGISTRY: OnceLock<Vec<AppEntry>> = OnceLock::new();

fn registry() -> &'static [AppEntry] {
    REGISTRY.get_or_init(|| {
        let file = RegistryAsset::get("registry.yaml").expect("registry.yaml embedded");
        serde_yaml::from_slice(&file.data).expect("valid registry.yaml")
    })
}

pub fn all_app_entries() -> &'static [AppEntry] { registry() }

pub fn minimal_set_identifiers() -> &'static [&'static str] { MINIMAL_SET }

pub fn all_apps_identifiers() -> Vec<&'static str> {
    let mut ids: Vec<&str> = registry().iter().map(|e| e.id.as_str()).collect();
    ids.sort_unstable();
    ids
}
