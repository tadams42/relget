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

pub trait App {
    fn exe_name(&self) -> &str;

    fn installed_version_flag(&self) -> &str { "--version" }

    fn released_version(&self) -> Result<AppVersion>;
    fn download(&self) -> Result<DownloadedAssets>;

    fn parse_installed_version(&self, data: &str) -> Option<AppVersion> {
        AppVersion::find_in(data)
    }

    fn installed_version(&self, prefix: &Path) -> Result<Option<AppVersion>> {
        let bin = prefix.join("bin").join(self.exe_name());
        if !bin.exists() {
            return Ok(None);
        }
        let out = std::process::Command::new(&bin)
            .arg(self.installed_version_flag())
            .output();
        match out {
            Err(_) => Ok(None),
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let stderr = String::from_utf8_lossy(&o.stderr);
                let combined = format!("{}{}", stdout, stderr);
                Ok(self.parse_installed_version(&combined))
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

pub fn all_app_entries() -> Vec<AppEntry> {
    vec![
        AppEntry {
            id:          "aqua",
            url:         dev_envs::aqua::Aqua::URL,
            category:    "dev_envs",
            description: dev_envs::aqua::Aqua::DESCRIPTION,
        },
        AppEntry {
            id:          "ast-grep",
            url:         dev_tools::ast_grep::AstGrep::URL,
            category:    "dev_tools",
            description: dev_tools::ast_grep::AstGrep::DESCRIPTION,
        },
        AppEntry {
            id:          "atuin",
            url:         shell::atuin::Atuin::URL,
            category:    "shell",
            description: shell::atuin::Atuin::DESCRIPTION,
        },
        AppEntry {
            id:          "bat",
            url:         files::bat::Bat::URL,
            category:    "files",
            description: files::bat::Bat::DESCRIPTION,
        },
        AppEntry {
            id:          "caddy",
            url:         http::caddy::Caddy::URL,
            category:    "http",
            description: http::caddy::Caddy::DESCRIPTION,
        },
        AppEntry {
            id:          "carapace",
            url:         shell::carapace::Carapace::URL,
            category:    "shell",
            description: shell::carapace::Carapace::DESCRIPTION,
        },
        AppEntry {
            id:          "chezmoi",
            url:         chezmoi::Chezmoi::URL,
            category:    "other",
            description: chezmoi::Chezmoi::DESCRIPTION,
        },
        AppEntry {
            id:          "d4s",
            url:         containers::d4s::D4S::URL,
            category:    "containers",
            description: containers::d4s::D4S::DESCRIPTION,
        },
        AppEntry {
            id:          "dasel",
            url:         data::dasel::Dasel::URL,
            category:    "data",
            description: data::dasel::Dasel::DESCRIPTION,
        },
        AppEntry {
            id:          "delta",
            url:         git::delta::Delta::URL,
            category:    "git",
            description: git::delta::Delta::DESCRIPTION,
        },
        AppEntry {
            id:          "difft",
            url:         git::difftastic::Difftastic::URL,
            category:    "git",
            description: git::difftastic::Difftastic::DESCRIPTION,
        },
        AppEntry {
            id:          "dockmate",
            url:         containers::dock_mate::DockMate::URL,
            category:    "containers",
            description: containers::dock_mate::DockMate::DESCRIPTION,
        },
        AppEntry {
            id:          "dry",
            url:         containers::dry::Dry::URL,
            category:    "containers",
            description: containers::dry::Dry::DESCRIPTION,
        },
        AppEntry {
            id:          "dust",
            url:         files::dust::Dust::URL,
            category:    "files",
            description: files::dust::Dust::DESCRIPTION,
        },
        AppEntry {
            id:          "eza",
            url:         files::eza::Eza::URL,
            category:    "files",
            description: files::eza::Eza::DESCRIPTION,
        },
        AppEntry {
            id:          "fd",
            url:         files::fd_find::FdFind::URL,
            category:    "files",
            description: files::fd_find::FdFind::DESCRIPTION,
        },
        AppEntry {
            id:          "fnm",
            url:         dev_envs::fnm::Fnm::URL,
            category:    "dev_envs",
            description: dev_envs::fnm::Fnm::DESCRIPTION,
        },
        AppEntry {
            id:          "fx",
            url:         data::fx::Fx::URL,
            category:    "data",
            description: data::fx::Fx::DESCRIPTION,
        },
        AppEntry {
            id:          "fzf",
            url:         shell::fzf::Fzf::URL,
            category:    "shell",
            description: shell::fzf::Fzf::DESCRIPTION,
        },
        AppEntry {
            id:          "gitleaks",
            url:         git::gitleaks::Gitleaks::URL,
            category:    "git",
            description: git::gitleaks::Gitleaks::DESCRIPTION,
        },
        AppEntry {
            id:          "gojq",
            url:         data::gojq::GoJq::URL,
            category:    "data",
            description: data::gojq::GoJq::DESCRIPTION,
        },
        AppEntry {
            id:          "gonzo",
            url:         logs::gonzo::Gonzo::URL,
            category:    "logs",
            description: logs::gonzo::Gonzo::DESCRIPTION,
        },
        AppEntry {
            id:          "jid",
            url:         data::jid::Jid::URL,
            category:    "data",
            description: data::jid::Jid::DESCRIPTION,
        },
        AppEntry {
            id:          "jq",
            url:         data::jq::Jq::URL,
            category:    "data",
            description: data::jq::Jq::DESCRIPTION,
        },
        AppEntry {
            id:          "jqp",
            url:         data::jqp::Jqp::URL,
            category:    "data",
            description: data::jqp::Jqp::DESCRIPTION,
        },
        AppEntry {
            id:          "lazyjournal",
            url:         logs::lazy_journal::LazyJournal::URL,
            category:    "logs",
            description: logs::lazy_journal::LazyJournal::DESCRIPTION,
        },
        AppEntry {
            id:          "lazydocker",
            url:         containers::lazydocker::LazyDocker::URL,
            category:    "containers",
            description: containers::lazydocker::LazyDocker::DESCRIPTION,
        },
        AppEntry {
            id:          "lazygit",
            url:         git::lazygit::Lazygit::URL,
            category:    "git",
            description: git::lazygit::Lazygit::DESCRIPTION,
        },
        AppEntry {
            id:          "mdbook",
            url:         dev_tools::mdbook::Mdbook::URL,
            category:    "dev_tools",
            description: dev_tools::mdbook::Mdbook::DESCRIPTION,
        },
        AppEntry {
            id:          "mergiraf",
            url:         git::mergiraf::Mergiraf::URL,
            category:    "git",
            description: git::mergiraf::Mergiraf::DESCRIPTION,
        },
        AppEntry {
            id:          "mise",
            url:         dev_envs::mise::Mise::URL,
            category:    "dev_envs",
            description: dev_envs::mise::Mise::DESCRIPTION,
        },
        AppEntry {
            id:          "neovide",
            url:         dev_tools::neovide::Neovide::URL,
            category:    "dev_tools",
            description: dev_tools::neovide::Neovide::DESCRIPTION,
        },
        AppEntry {
            id:          "rclone",
            url:         rclone::Rclone::URL,
            category:    "other",
            description: rclone::Rclone::DESCRIPTION,
        },
        AppEntry {
            id:          "restish",
            url:         http::restish::Restish::URL,
            category:    "http",
            description: http::restish::Restish::DESCRIPTION,
        },
        AppEntry {
            id:          "rg",
            url:         files::ripgrep::Ripgrep::URL,
            category:    "files",
            description: files::ripgrep::Ripgrep::DESCRIPTION,
        },
        AppEntry {
            id:          "qsv",
            url:         data::qsv::Qsv::URL,
            category:    "data",
            description: data::qsv::Qsv::DESCRIPTION,
        },
        AppEntry {
            id:          "qsv-all",
            url:         data::qsv_all::QsvAll::URL,
            category:    "data",
            description: data::qsv_all::QsvAll::DESCRIPTION,
        },
        AppEntry {
            id:          "rsv",
            url:         data::rsv::Rsv::URL,
            category:    "data",
            description: data::rsv::Rsv::DESCRIPTION,
        },
        AppEntry {
            id:          "rust-analyzer",
            url:         dev_tools::rust_analyzer::RustAnalyzer::URL,
            category:    "dev_tools",
            description: dev_tools::rust_analyzer::RustAnalyzer::DESCRIPTION,
        },
        AppEntry {
            id:          "sd",
            url:         files::sd_edit::SdEdit::URL,
            category:    "files",
            description: files::sd_edit::SdEdit::DESCRIPTION,
        },
        AppEntry {
            id:          "sk",
            url:         shell::skim::Skim::URL,
            category:    "shell",
            description: shell::skim::Skim::DESCRIPTION,
        },
        AppEntry {
            id:          "starship",
            url:         shell::starship::Starship::URL,
            category:    "shell",
            description: shell::starship::Starship::DESCRIPTION,
        },
        AppEntry {
            id:          "stylua",
            url:         dev_tools::stylua::Stylua::URL,
            category:    "dev_tools",
            description: dev_tools::stylua::Stylua::DESCRIPTION,
        },
        AppEntry {
            id:          "uv",
            url:         dev_envs::uv::Uv::URL,
            category:    "dev_envs",
            description: dev_envs::uv::Uv::DESCRIPTION,
        },
        AppEntry {
            id:          "xh",
            url:         http::xh::Xh::URL,
            category:    "http",
            description: http::xh::Xh::DESCRIPTION,
        },
        AppEntry {
            id:          "xq",
            url:         data::xq::Xq::URL,
            category:    "data",
            description: data::xq::Xq::DESCRIPTION,
        },
        AppEntry {
            id:          "yazi",
            url:         files::yazi::Yazi::URL,
            category:    "files",
            description: files::yazi::Yazi::DESCRIPTION,
        },
        AppEntry {
            id:          "yq",
            url:         data::yq::Yq::URL,
            category:    "data",
            description: data::yq::Yq::DESCRIPTION,
        },
        AppEntry {
            id:          "zoxide",
            url:         shell::zoxide::Zoxide::URL,
            category:    "shell",
            description: shell::zoxide::Zoxide::DESCRIPTION,
        },
    ]
}

pub fn create_app(
    id: &str, gh_token: Option<String>, cb_token: Option<String>, offline: bool,
) -> Option<Box<dyn App>> {
    use crate::codeberg::CodebergClient;
    use crate::github::GithubClient;
    use std::sync::Arc;
    let client = Arc::new(GithubClient::new(gh_token, offline));
    match id {
        "aqua" => Some(Box::new(dev_envs::aqua::Aqua::new(client))),
        "ast-grep" => Some(Box::new(dev_tools::ast_grep::AstGrep::new(client))),
        "atuin" => Some(Box::new(shell::atuin::Atuin::new(client))),
        "bat" => Some(Box::new(files::bat::Bat::new(client))),
        "caddy" => Some(Box::new(http::caddy::Caddy::new(client))),
        "carapace" => Some(Box::new(shell::carapace::Carapace::new(client))),
        "chezmoi" => Some(Box::new(chezmoi::Chezmoi::new(client))),
        "d4s" => Some(Box::new(containers::d4s::D4S::new(client))),
        "dasel" => Some(Box::new(data::dasel::Dasel::new(client))),
        "delta" => Some(Box::new(git::delta::Delta::new(client))),
        "difft" => Some(Box::new(git::difftastic::Difftastic::new(client))),
        "dockmate" => Some(Box::new(containers::dock_mate::DockMate::new(client))),
        "dry" => Some(Box::new(containers::dry::Dry::new(client))),
        "dust" => Some(Box::new(files::dust::Dust::new(client))),
        "eza" => Some(Box::new(files::eza::Eza::new(client))),
        "fd" => Some(Box::new(files::fd_find::FdFind::new(client))),
        "fnm" => Some(Box::new(dev_envs::fnm::Fnm::new(client))),
        "fx" => Some(Box::new(data::fx::Fx::new(client))),
        "fzf" => Some(Box::new(shell::fzf::Fzf::new(client))),
        "gitleaks" => Some(Box::new(git::gitleaks::Gitleaks::new(client))),
        "gojq" => Some(Box::new(data::gojq::GoJq::new(client))),
        "gonzo" => Some(Box::new(logs::gonzo::Gonzo::new(client))),
        "jid" => Some(Box::new(data::jid::Jid::new(client))),
        "jq" => Some(Box::new(data::jq::Jq::new(client))),
        "jqp" => Some(Box::new(data::jqp::Jqp::new(client))),
        "lazyjournal" => Some(Box::new(logs::lazy_journal::LazyJournal::new(client))),
        "lazydocker" => Some(Box::new(containers::lazydocker::LazyDocker::new(client))),
        "lazygit" => Some(Box::new(git::lazygit::Lazygit::new(client))),
        "mdbook" => Some(Box::new(dev_tools::mdbook::Mdbook::new(client))),
        "mergiraf" => {
            Some(Box::new(git::mergiraf::Mergiraf::new(Arc::new(CodebergClient::new(
                cb_token, offline,
            )))))
        }
        "mise" => Some(Box::new(dev_envs::mise::Mise::new(client))),
        "neovide" => Some(Box::new(dev_tools::neovide::Neovide::new(client))),
        "rclone" => Some(Box::new(rclone::Rclone::new(client))),
        "restish" => Some(Box::new(http::restish::Restish::new(client))),
        "rg" => Some(Box::new(files::ripgrep::Ripgrep::new(client))),
        "qsv" => Some(Box::new(data::qsv::Qsv::new(client))),
        "qsv-all" => Some(Box::new(data::qsv_all::QsvAll::new(client))),
        "rsv" => Some(Box::new(data::rsv::Rsv::new(client))),
        "rust-analyzer" => Some(Box::new(dev_tools::rust_analyzer::RustAnalyzer::new(client))),
        "sd" => Some(Box::new(files::sd_edit::SdEdit::new(client))),
        "sk" => Some(Box::new(shell::skim::Skim::new(client))),
        "starship" => Some(Box::new(shell::starship::Starship::new(client))),
        "stylua" => Some(Box::new(dev_tools::stylua::Stylua::new(client))),
        "uv" => Some(Box::new(dev_envs::uv::Uv::new(client))),
        "xh" => Some(Box::new(http::xh::Xh::new(client))),
        "xq" => Some(Box::new(data::xq::Xq::new(client))),
        "yazi" => Some(Box::new(files::yazi::Yazi::new(client))),
        "yq" => Some(Box::new(data::yq::Yq::new(client))),
        "zoxide" => Some(Box::new(shell::zoxide::Zoxide::new(client))),
        _ => None,
    }
}
