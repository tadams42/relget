use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::installer::install_assets;
use crate::types::DownloadedAssets;
use crate::version::AppVersion;

pub mod chezmoi;
pub mod containers;
pub mod data;
pub mod dev_envs;
pub mod dev_tools;
pub mod files;
pub mod git;
pub mod http;
pub mod logs;
pub mod rclone;
pub mod shell;

// ── App trait ────────────────────────────────────────────────────────────────

const DEFAULT_VERSION_ARG: &str = "--version";

pub trait App {
    fn exe_name(&self) -> &str;

    fn cli_version_arg(&self) -> &str { DEFAULT_VERSION_ARG }

    fn released_version(&self) -> Result<AppVersion>;

    fn download(&self) -> Result<DownloadedAssets>;

    fn installed_version(&self, prefix: &Path) -> Result<Option<AppVersion>> {
        let bin = prefix.join("bin").join(self.exe_name());
        if !bin.exists() {
            return Ok(None);
        }
        let out = std::process::Command::new(&bin)
            .arg(self.cli_version_arg())
            .output();
        match out {
            Err(_) => Ok(None),
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let stderr = String::from_utf8_lossy(&o.stderr);
                let combined = format!("{}{}", stdout, stderr);
                Ok(AppVersion::find_in(&combined))
            }
        }
    }

    fn needs_install(&self, prefix: &Path) -> Result<bool> {
        let installed = self.installed_version(prefix)?;
        match installed {
            None => Ok(true),
            Some(iv) => Ok(iv != self.released_version()?),
        }
    }

    fn install(&self, prefix: &Path) -> Result<Vec<PathBuf>> {
        if !self.needs_install(prefix)? {
            log::info!("lvl=INFO app={} msg=Already at latest version", self.exe_name());
            return Ok(vec![]);
        }

        let assets = self.download()?;
        let installed = install_assets(prefix, &assets)?;
        log::info!("lvl=INFO app={} msg=Installed", self.exe_name());
        Ok(installed)
    }
}

// ── Registry ─────────────────────────────────────────────────────────────────

pub struct AppEntry {
    pub id:          &'static str,
    pub url:         &'static str,
    pub category:    &'static str,
    pub description: &'static str,
}

static ALL_APP_ENTRIES: &[AppEntry] = &[
    AppEntry {
        id:          dev_envs::aqua::Aqua::ID,
        url:         dev_envs::aqua::Aqua::URL,
        category:    "dev_envs",
        description: dev_envs::aqua::Aqua::DESCRIPTION,
    },
    AppEntry {
        id:          dev_tools::ast_grep::AstGrep::ID,
        url:         dev_tools::ast_grep::AstGrep::URL,
        category:    "dev_tools",
        description: dev_tools::ast_grep::AstGrep::DESCRIPTION,
    },
    AppEntry {
        id:          shell::atuin::Atuin::ID,
        url:         shell::atuin::Atuin::URL,
        category:    "shell",
        description: shell::atuin::Atuin::DESCRIPTION,
    },
    AppEntry {
        id:          files::bat::Bat::ID,
        url:         files::bat::Bat::URL,
        category:    "files",
        description: files::bat::Bat::DESCRIPTION,
    },
    AppEntry {
        id:          http::caddy::Caddy::ID,
        url:         http::caddy::Caddy::URL,
        category:    "http",
        description: http::caddy::Caddy::DESCRIPTION,
    },
    AppEntry {
        id:          shell::carapace::Carapace::ID,
        url:         shell::carapace::Carapace::URL,
        category:    "shell",
        description: shell::carapace::Carapace::DESCRIPTION,
    },
    AppEntry {
        id:          chezmoi::Chezmoi::ID,
        url:         chezmoi::Chezmoi::URL,
        category:    "other",
        description: chezmoi::Chezmoi::DESCRIPTION,
    },
    AppEntry {
        id:          containers::d4s::D4S::ID,
        url:         containers::d4s::D4S::URL,
        category:    "containers",
        description: containers::d4s::D4S::DESCRIPTION,
    },
    AppEntry {
        id:          data::dasel::Dasel::ID,
        url:         data::dasel::Dasel::URL,
        category:    "data",
        description: data::dasel::Dasel::DESCRIPTION,
    },
    AppEntry {
        id:          git::delta::Delta::ID,
        url:         git::delta::Delta::URL,
        category:    "git",
        description: git::delta::Delta::DESCRIPTION,
    },
    AppEntry {
        id:          git::difftastic::Difftastic::ID,
        url:         git::difftastic::Difftastic::URL,
        category:    "git",
        description: git::difftastic::Difftastic::DESCRIPTION,
    },
    AppEntry {
        id:          containers::dock_mate::DockMate::ID,
        url:         containers::dock_mate::DockMate::URL,
        category:    "containers",
        description: containers::dock_mate::DockMate::DESCRIPTION,
    },
    AppEntry {
        id:          containers::dry::Dry::ID,
        url:         containers::dry::Dry::URL,
        category:    "containers",
        description: containers::dry::Dry::DESCRIPTION,
    },
    AppEntry {
        id:          files::dust::Dust::ID,
        url:         files::dust::Dust::URL,
        category:    "files",
        description: files::dust::Dust::DESCRIPTION,
    },
    AppEntry {
        id:          files::eza::Eza::ID,
        url:         files::eza::Eza::URL,
        category:    "files",
        description: files::eza::Eza::DESCRIPTION,
    },
    AppEntry {
        id:          files::fd_find::FdFind::ID,
        url:         files::fd_find::FdFind::URL,
        category:    "files",
        description: files::fd_find::FdFind::DESCRIPTION,
    },
    AppEntry {
        id:          dev_envs::fnm::Fnm::ID,
        url:         dev_envs::fnm::Fnm::URL,
        category:    "dev_envs",
        description: dev_envs::fnm::Fnm::DESCRIPTION,
    },
    AppEntry {
        id:          data::fx::Fx::ID,
        url:         data::fx::Fx::URL,
        category:    "data",
        description: data::fx::Fx::DESCRIPTION,
    },
    AppEntry {
        id:          shell::fzf::Fzf::ID,
        url:         shell::fzf::Fzf::URL,
        category:    "shell",
        description: shell::fzf::Fzf::DESCRIPTION,
    },
    AppEntry {
        id:          git::gitleaks::Gitleaks::ID,
        url:         git::gitleaks::Gitleaks::URL,
        category:    "git",
        description: git::gitleaks::Gitleaks::DESCRIPTION,
    },
    AppEntry {
        id:          data::gojq::GoJq::ID,
        url:         data::gojq::GoJq::URL,
        category:    "data",
        description: data::gojq::GoJq::DESCRIPTION,
    },
    AppEntry {
        id:          logs::gonzo::Gonzo::ID,
        url:         logs::gonzo::Gonzo::URL,
        category:    "logs",
        description: logs::gonzo::Gonzo::DESCRIPTION,
    },
    AppEntry {
        id:          data::jid::Jid::ID,
        url:         data::jid::Jid::URL,
        category:    "data",
        description: data::jid::Jid::DESCRIPTION,
    },
    AppEntry {
        id:          data::jq::Jq::ID,
        url:         data::jq::Jq::URL,
        category:    "data",
        description: data::jq::Jq::DESCRIPTION,
    },
    AppEntry {
        id:          data::jqp::Jqp::ID,
        url:         data::jqp::Jqp::URL,
        category:    "data",
        description: data::jqp::Jqp::DESCRIPTION,
    },
    AppEntry {
        id:          logs::lazy_journal::LazyJournal::ID,
        url:         logs::lazy_journal::LazyJournal::URL,
        category:    "logs",
        description: logs::lazy_journal::LazyJournal::DESCRIPTION,
    },
    AppEntry {
        id:          containers::lazydocker::LazyDocker::ID,
        url:         containers::lazydocker::LazyDocker::URL,
        category:    "containers",
        description: containers::lazydocker::LazyDocker::DESCRIPTION,
    },
    AppEntry {
        id:          git::lazygit::Lazygit::ID,
        url:         git::lazygit::Lazygit::URL,
        category:    "git",
        description: git::lazygit::Lazygit::DESCRIPTION,
    },
    AppEntry {
        id:          dev_tools::mdbook::Mdbook::ID,
        url:         dev_tools::mdbook::Mdbook::URL,
        category:    "dev_tools",
        description: dev_tools::mdbook::Mdbook::DESCRIPTION,
    },
    AppEntry {
        id:          git::mergiraf::Mergiraf::ID,
        url:         git::mergiraf::Mergiraf::URL,
        category:    "git",
        description: git::mergiraf::Mergiraf::DESCRIPTION,
    },
    AppEntry {
        id:          dev_envs::mise::Mise::ID,
        url:         dev_envs::mise::Mise::URL,
        category:    "dev_envs",
        description: dev_envs::mise::Mise::DESCRIPTION,
    },
    AppEntry {
        id:          dev_tools::neovide::Neovide::ID,
        url:         dev_tools::neovide::Neovide::URL,
        category:    "dev_tools",
        description: dev_tools::neovide::Neovide::DESCRIPTION,
    },
    AppEntry {
        id:          rclone::Rclone::ID,
        url:         rclone::Rclone::URL,
        category:    "other",
        description: rclone::Rclone::DESCRIPTION,
    },
    AppEntry {
        id:          http::restish::Restish::ID,
        url:         http::restish::Restish::URL,
        category:    "http",
        description: http::restish::Restish::DESCRIPTION,
    },
    AppEntry {
        id:          files::ripgrep::Ripgrep::ID,
        url:         files::ripgrep::Ripgrep::URL,
        category:    "files",
        description: files::ripgrep::Ripgrep::DESCRIPTION,
    },
    AppEntry {
        id:          data::qsv::Qsv::ID,
        url:         data::qsv::Qsv::URL,
        category:    "data",
        description: data::qsv::Qsv::DESCRIPTION,
    },
    AppEntry {
        id:          data::qsv_all::QsvAll::ID,
        url:         data::qsv_all::QsvAll::URL,
        category:    "data",
        description: data::qsv_all::QsvAll::DESCRIPTION,
    },
    AppEntry {
        id:          data::rsv::Rsv::ID,
        url:         data::rsv::Rsv::URL,
        category:    "data",
        description: data::rsv::Rsv::DESCRIPTION,
    },
    AppEntry {
        id:          dev_tools::rust_analyzer::RustAnalyzer::ID,
        url:         dev_tools::rust_analyzer::RustAnalyzer::URL,
        category:    "dev_tools",
        description: dev_tools::rust_analyzer::RustAnalyzer::DESCRIPTION,
    },
    AppEntry {
        id:          files::sd_edit::SdEdit::ID,
        url:         files::sd_edit::SdEdit::URL,
        category:    "files",
        description: files::sd_edit::SdEdit::DESCRIPTION,
    },
    AppEntry {
        id:          shell::skim::Skim::ID,
        url:         shell::skim::Skim::URL,
        category:    "shell",
        description: shell::skim::Skim::DESCRIPTION,
    },
    AppEntry {
        id:          shell::starship::Starship::ID,
        url:         shell::starship::Starship::URL,
        category:    "shell",
        description: shell::starship::Starship::DESCRIPTION,
    },
    AppEntry {
        id:          dev_tools::stylua::Stylua::ID,
        url:         dev_tools::stylua::Stylua::URL,
        category:    "dev_tools",
        description: dev_tools::stylua::Stylua::DESCRIPTION,
    },
    AppEntry {
        id:          dev_envs::uv::Uv::ID,
        url:         dev_envs::uv::Uv::URL,
        category:    "dev_envs",
        description: dev_envs::uv::Uv::DESCRIPTION,
    },
    AppEntry {
        id:          http::xh::Xh::ID,
        url:         http::xh::Xh::URL,
        category:    "http",
        description: http::xh::Xh::DESCRIPTION,
    },
    AppEntry {
        id:          data::xq::Xq::ID,
        url:         data::xq::Xq::URL,
        category:    "data",
        description: data::xq::Xq::DESCRIPTION,
    },
    AppEntry {
        id:          files::yazi::Yazi::ID,
        url:         files::yazi::Yazi::URL,
        category:    "files",
        description: files::yazi::Yazi::DESCRIPTION,
    },
    AppEntry {
        id:          data::yq::Yq::ID,
        url:         data::yq::Yq::URL,
        category:    "data",
        description: data::yq::Yq::DESCRIPTION,
    },
    AppEntry {
        id:          shell::zoxide::Zoxide::ID,
        url:         shell::zoxide::Zoxide::URL,
        category:    "shell",
        description: shell::zoxide::Zoxide::DESCRIPTION,
    },
];

pub fn all_app_entries() -> &'static [AppEntry] { ALL_APP_ENTRIES }

pub fn create_app(
    id: &str, gh_token: Option<String>, cb_token: Option<String>, offline: bool,
) -> Option<Box<dyn App>> {
    use crate::codeberg::CodebergClient;
    use crate::github::GithubClient;
    use std::sync::Arc;
    let client = Arc::new(GithubClient::new(gh_token, offline));
    match id {
        dev_envs::aqua::Aqua::ID => Some(Box::new(dev_envs::aqua::Aqua::new(client))),
        dev_tools::ast_grep::AstGrep::ID => {
            Some(Box::new(dev_tools::ast_grep::AstGrep::new(client)))
        }
        shell::atuin::Atuin::ID => Some(Box::new(shell::atuin::Atuin::new(client))),
        files::bat::Bat::ID => Some(Box::new(files::bat::Bat::new(client))),
        http::caddy::Caddy::ID => Some(Box::new(http::caddy::Caddy::new(client))),
        shell::carapace::Carapace::ID => Some(Box::new(shell::carapace::Carapace::new(client))),
        chezmoi::Chezmoi::ID => Some(Box::new(chezmoi::Chezmoi::new(client))),
        containers::d4s::D4S::ID => Some(Box::new(containers::d4s::D4S::new(client))),
        data::dasel::Dasel::ID => Some(Box::new(data::dasel::Dasel::new(client))),
        git::delta::Delta::ID => Some(Box::new(git::delta::Delta::new(client))),
        git::difftastic::Difftastic::ID => Some(Box::new(git::difftastic::Difftastic::new(client))),
        containers::dock_mate::DockMate::ID => {
            Some(Box::new(containers::dock_mate::DockMate::new(client)))
        }
        containers::dry::Dry::ID => Some(Box::new(containers::dry::Dry::new(client))),
        files::dust::Dust::ID => Some(Box::new(files::dust::Dust::new(client))),
        files::eza::Eza::ID => Some(Box::new(files::eza::Eza::new(client))),
        files::fd_find::FdFind::ID => Some(Box::new(files::fd_find::FdFind::new(client))),
        dev_envs::fnm::Fnm::ID => Some(Box::new(dev_envs::fnm::Fnm::new(client))),
        data::fx::Fx::ID => Some(Box::new(data::fx::Fx::new(client))),
        shell::fzf::Fzf::ID => Some(Box::new(shell::fzf::Fzf::new(client))),
        git::gitleaks::Gitleaks::ID => Some(Box::new(git::gitleaks::Gitleaks::new(client))),
        data::gojq::GoJq::ID => Some(Box::new(data::gojq::GoJq::new(client))),
        logs::gonzo::Gonzo::ID => Some(Box::new(logs::gonzo::Gonzo::new(client))),
        data::jid::Jid::ID => Some(Box::new(data::jid::Jid::new(client))),
        data::jq::Jq::ID => Some(Box::new(data::jq::Jq::new(client))),
        data::jqp::Jqp::ID => Some(Box::new(data::jqp::Jqp::new(client))),
        logs::lazy_journal::LazyJournal::ID => {
            Some(Box::new(logs::lazy_journal::LazyJournal::new(client)))
        }
        containers::lazydocker::LazyDocker::ID => {
            Some(Box::new(containers::lazydocker::LazyDocker::new(client)))
        }
        git::lazygit::Lazygit::ID => Some(Box::new(git::lazygit::Lazygit::new(client))),
        dev_tools::mdbook::Mdbook::ID => Some(Box::new(dev_tools::mdbook::Mdbook::new(client))),
        git::mergiraf::Mergiraf::ID => {
            Some(Box::new(git::mergiraf::Mergiraf::new(Arc::new(CodebergClient::new(
                cb_token, offline,
            )))))
        }
        dev_envs::mise::Mise::ID => Some(Box::new(dev_envs::mise::Mise::new(client))),
        dev_tools::neovide::Neovide::ID => Some(Box::new(dev_tools::neovide::Neovide::new(client))),
        rclone::Rclone::ID => Some(Box::new(rclone::Rclone::new(client))),
        http::restish::Restish::ID => Some(Box::new(http::restish::Restish::new(client))),
        files::ripgrep::Ripgrep::ID => Some(Box::new(files::ripgrep::Ripgrep::new(client))),
        data::qsv::Qsv::ID => Some(Box::new(data::qsv::Qsv::new(client))),
        data::qsv_all::QsvAll::ID => Some(Box::new(data::qsv_all::QsvAll::new(client))),
        data::rsv::Rsv::ID => Some(Box::new(data::rsv::Rsv::new(client))),
        dev_tools::rust_analyzer::RustAnalyzer::ID => {
            Some(Box::new(dev_tools::rust_analyzer::RustAnalyzer::new(client)))
        }
        files::sd_edit::SdEdit::ID => Some(Box::new(files::sd_edit::SdEdit::new(client))),
        shell::skim::Skim::ID => Some(Box::new(shell::skim::Skim::new(client))),
        shell::starship::Starship::ID => Some(Box::new(shell::starship::Starship::new(client))),
        dev_tools::stylua::Stylua::ID => Some(Box::new(dev_tools::stylua::Stylua::new(client))),
        dev_envs::uv::Uv::ID => Some(Box::new(dev_envs::uv::Uv::new(client))),
        http::xh::Xh::ID => Some(Box::new(http::xh::Xh::new(client))),
        data::xq::Xq::ID => Some(Box::new(data::xq::Xq::new(client))),
        files::yazi::Yazi::ID => Some(Box::new(files::yazi::Yazi::new(client))),
        data::yq::Yq::ID => Some(Box::new(data::yq::Yq::new(client))),
        shell::zoxide::Zoxide::ID => Some(Box::new(shell::zoxide::Zoxide::new(client))),
        _ => None,
    }
}
