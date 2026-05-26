use super::containers::{D4S, DockMate, Dry, LazyDocker};
use super::data::{Dasel, Fx, GoJq, Jid, Jq, Jqp, Qsv, QsvAll, Rsv, Xq, Yq};
use super::dev_envs::{Aqua, Fnm, Mise, Uv};
use super::dev_tools::{AstGrep, Mdbook, Neovide, RustAnalyzer, Scc, Stylua};
use super::files::{Bat, Dust, Eza, F2, FdFind, Ripgrep, SdEdit, Trash, Yazi};
use super::git::{Delta, Difftastic, Gitleaks, Lazygit, Mergiraf};
use super::http::{Caddy, Restish, Xh};
use super::logs::{Gonzo, LazyJournal};
use super::music::Spotatui;
use super::other::{Chezmoi, Rclone};
use super::shell::{Atuin, Carapace, Fzf, Skim, Starship, Zoxide};

#[rustfmt::skip]
pub const MINIMAL_SET: &[&str] = &[
    Bat::ID,
    Chezmoi::ID,
    D4S::ID,
    Delta::ID,
    Difftastic::ID,
    Dust::ID,
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

pub struct AppEntry {
    pub id:          &'static str,
    pub url:         &'static str,
    pub category:    &'static str,
    pub description: &'static str,
}

static ALL_APP_ENTRIES: &[AppEntry] = &[
    AppEntry {
        id:          Aqua::ID,
        url:         Aqua::URL,
        category:    "dev_envs",
        description: Aqua::DESCRIPTION,
    },
    AppEntry {
        id:          AstGrep::ID,
        url:         AstGrep::URL,
        category:    "dev_tools",
        description: AstGrep::DESCRIPTION,
    },
    AppEntry {
        id:          Atuin::ID,
        url:         Atuin::URL,
        category:    "shell",
        description: Atuin::DESCRIPTION,
    },
    AppEntry {
        id:          Bat::ID,
        url:         Bat::URL,
        category:    "files",
        description: Bat::DESCRIPTION,
    },
    AppEntry {
        id:          Caddy::ID,
        url:         Caddy::URL,
        category:    "http",
        description: Caddy::DESCRIPTION,
    },
    AppEntry {
        id:          Carapace::ID,
        url:         Carapace::URL,
        category:    "shell",
        description: Carapace::DESCRIPTION,
    },
    AppEntry {
        id:          Chezmoi::ID,
        url:         Chezmoi::URL,
        category:    "other",
        description: Chezmoi::DESCRIPTION,
    },
    AppEntry {
        id:          D4S::ID,
        url:         D4S::URL,
        category:    "containers",
        description: D4S::DESCRIPTION,
    },
    AppEntry {
        id:          Dasel::ID,
        url:         Dasel::URL,
        category:    "data",
        description: Dasel::DESCRIPTION,
    },
    AppEntry {
        id:          Delta::ID,
        url:         Delta::URL,
        category:    "git",
        description: Delta::DESCRIPTION,
    },
    AppEntry {
        id:          Difftastic::ID,
        url:         Difftastic::URL,
        category:    "git",
        description: Difftastic::DESCRIPTION,
    },
    AppEntry {
        id:          DockMate::ID,
        url:         DockMate::URL,
        category:    "containers",
        description: DockMate::DESCRIPTION,
    },
    AppEntry {
        id:          Dry::ID,
        url:         Dry::URL,
        category:    "containers",
        description: Dry::DESCRIPTION,
    },
    AppEntry {
        id:          Dust::ID,
        url:         Dust::URL,
        category:    "files",
        description: Dust::DESCRIPTION,
    },
    AppEntry {
        id:          Eza::ID,
        url:         Eza::URL,
        category:    "files",
        description: Eza::DESCRIPTION,
    },
    AppEntry {
        id:          F2::ID,
        url:         F2::URL,
        category:    "files",
        description: F2::DESCRIPTION,
    },
    AppEntry {
        id:          FdFind::ID,
        url:         FdFind::URL,
        category:    "files",
        description: FdFind::DESCRIPTION,
    },
    AppEntry {
        id:          Fnm::ID,
        url:         Fnm::URL,
        category:    "dev_envs",
        description: Fnm::DESCRIPTION,
    },
    AppEntry {
        id:          Fx::ID,
        url:         Fx::URL,
        category:    "data",
        description: Fx::DESCRIPTION,
    },
    AppEntry {
        id:          Fzf::ID,
        url:         Fzf::URL,
        category:    "shell",
        description: Fzf::DESCRIPTION,
    },
    AppEntry {
        id:          Gitleaks::ID,
        url:         Gitleaks::URL,
        category:    "git",
        description: Gitleaks::DESCRIPTION,
    },
    AppEntry {
        id:          GoJq::ID,
        url:         GoJq::URL,
        category:    "data",
        description: GoJq::DESCRIPTION,
    },
    AppEntry {
        id:          Gonzo::ID,
        url:         Gonzo::URL,
        category:    "logs",
        description: Gonzo::DESCRIPTION,
    },
    AppEntry {
        id:          Jid::ID,
        url:         Jid::URL,
        category:    "data",
        description: Jid::DESCRIPTION,
    },
    AppEntry {
        id:          Jq::ID,
        url:         Jq::URL,
        category:    "data",
        description: Jq::DESCRIPTION,
    },
    AppEntry {
        id:          Jqp::ID,
        url:         Jqp::URL,
        category:    "data",
        description: Jqp::DESCRIPTION,
    },
    AppEntry {
        id:          LazyJournal::ID,
        url:         LazyJournal::URL,
        category:    "logs",
        description: LazyJournal::DESCRIPTION,
    },
    AppEntry {
        id:          LazyDocker::ID,
        url:         LazyDocker::URL,
        category:    "containers",
        description: LazyDocker::DESCRIPTION,
    },
    AppEntry {
        id:          Lazygit::ID,
        url:         Lazygit::URL,
        category:    "git",
        description: Lazygit::DESCRIPTION,
    },
    AppEntry {
        id:          Mdbook::ID,
        url:         Mdbook::URL,
        category:    "dev_tools",
        description: Mdbook::DESCRIPTION,
    },
    AppEntry {
        id:          Mergiraf::ID,
        url:         Mergiraf::URL,
        category:    "git",
        description: Mergiraf::DESCRIPTION,
    },
    AppEntry {
        id:          Mise::ID,
        url:         Mise::URL,
        category:    "dev_envs",
        description: Mise::DESCRIPTION,
    },
    AppEntry {
        id:          Neovide::ID,
        url:         Neovide::URL,
        category:    "dev_tools",
        description: Neovide::DESCRIPTION,
    },
    AppEntry {
        id:          Rclone::ID,
        url:         Rclone::URL,
        category:    "other",
        description: Rclone::DESCRIPTION,
    },
    AppEntry {
        id:          Restish::ID,
        url:         Restish::URL,
        category:    "http",
        description: Restish::DESCRIPTION,
    },
    AppEntry {
        id:          Ripgrep::ID,
        url:         Ripgrep::URL,
        category:    "files",
        description: Ripgrep::DESCRIPTION,
    },
    AppEntry {
        id:          Qsv::ID,
        url:         Qsv::URL,
        category:    "data",
        description: Qsv::DESCRIPTION,
    },
    AppEntry {
        id:          QsvAll::ID,
        url:         QsvAll::URL,
        category:    "data",
        description: QsvAll::DESCRIPTION,
    },
    AppEntry {
        id:          Rsv::ID,
        url:         Rsv::URL,
        category:    "data",
        description: Rsv::DESCRIPTION,
    },
    AppEntry {
        id:          RustAnalyzer::ID,
        url:         RustAnalyzer::URL,
        category:    "dev_tools",
        description: RustAnalyzer::DESCRIPTION,
    },
    AppEntry {
        id:          Scc::ID,
        url:         Scc::URL,
        category:    "dev_tools",
        description: Scc::DESCRIPTION,
    },
    AppEntry {
        id:          SdEdit::ID,
        url:         SdEdit::URL,
        category:    "files",
        description: SdEdit::DESCRIPTION,
    },
    AppEntry {
        id:          Skim::ID,
        url:         Skim::URL,
        category:    "shell",
        description: Skim::DESCRIPTION,
    },
    AppEntry {
        id:          Spotatui::ID,
        url:         Spotatui::URL,
        category:    "music",
        description: Spotatui::DESCRIPTION,
    },
    AppEntry {
        id:          Starship::ID,
        url:         Starship::URL,
        category:    "shell",
        description: Starship::DESCRIPTION,
    },
    AppEntry {
        id:          Stylua::ID,
        url:         Stylua::URL,
        category:    "dev_tools",
        description: Stylua::DESCRIPTION,
    },
    AppEntry {
        id:          Trash::ID,
        url:         Trash::URL,
        category:    "files",
        description: Trash::DESCRIPTION,
    },
    AppEntry {
        id:          Uv::ID,
        url:         Uv::URL,
        category:    "dev_envs",
        description: Uv::DESCRIPTION,
    },
    AppEntry {
        id:          Xh::ID,
        url:         Xh::URL,
        category:    "http",
        description: Xh::DESCRIPTION,
    },
    AppEntry {
        id:          Xq::ID,
        url:         Xq::URL,
        category:    "data",
        description: Xq::DESCRIPTION,
    },
    AppEntry {
        id:          Yazi::ID,
        url:         Yazi::URL,
        category:    "files",
        description: Yazi::DESCRIPTION,
    },
    AppEntry {
        id:          Yq::ID,
        url:         Yq::URL,
        category:    "data",
        description: Yq::DESCRIPTION,
    },
    AppEntry {
        id:          Zoxide::ID,
        url:         Zoxide::URL,
        category:    "shell",
        description: Zoxide::DESCRIPTION,
    },
];

pub fn all_app_entries() -> &'static [AppEntry] { ALL_APP_ENTRIES }

pub fn minimal_set_identifiers() -> &'static [&'static str] { MINIMAL_SET }

pub fn all_apps_identifiers() -> Vec<&'static str> {
    let mut ids: Vec<&'static str> = ALL_APP_ENTRIES.iter().map(|e| e.id).collect();
    ids.sort_unstable();
    ids
}
