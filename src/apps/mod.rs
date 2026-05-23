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
    fn url(&self) -> &str;

    fn installed_version_flag(&self) -> &str { "--version" }

    /// Index into the whitespace-split output of `<exe> --version`.
    /// Negative means from the end (Python-style).
    fn installed_version_word_index(&self) -> isize { -1 }

    fn released_version(&self) -> Result<AppVersion>;
    fn download(&self) -> Result<DownloadedAssets>;

    fn parse_installed_version(&self, data: &str) -> Option<AppVersion> {
        let normalized = data.replace(',', " ");
        let words: Vec<&str> = normalized.split_whitespace().collect();
        if words.is_empty() {
            return None;
        }
        let idx = self.installed_version_word_index();
        let word = if idx < 0 {
            let abs = (-idx) as usize;
            words.get(words.len().wrapping_sub(abs)).copied()?
        } else {
            words.get(idx as usize).copied()?
        };
        AppVersion::parse(word)
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
    pub id:       &'static str,
    pub url:      &'static str,
    pub category: &'static str,
}

pub fn all_app_entries() -> Vec<AppEntry> {
    vec![
        AppEntry {
            id:       "aqua",
            url:      "https://github.com/aquaproj/aqua",
            category: "dev_envs",
        },
        AppEntry {
            id:       "ast-grep",
            url:      "https://github.com/ast-grep/ast-grep",
            category: "dev_tools",
        },
        AppEntry {
            id:       "atuin",
            url:      "https://github.com/atuinsh/atuin",
            category: "shell",
        },
        AppEntry {
            id:       "bat",
            url:      "https://github.com/sharkdp/bat",
            category: "files",
        },
        AppEntry {
            id:       "caddy",
            url:      "https://github.com/caddyserver/caddy",
            category: "http",
        },
        AppEntry {
            id:       "carapace",
            url:      "https://github.com/carapace-sh/carapace-bin",
            category: "shell",
        },
        AppEntry {
            id:       "chezmoi",
            url:      "https://github.com/twpayne/chezmoi",
            category: "other",
        },
        AppEntry {
            id:       "d4s",
            url:      "https://github.com/jr-k/d4s",
            category: "containers",
        },
        AppEntry {
            id:       "dasel",
            url:      "https://github.com/TomWright/dasel",
            category: "data",
        },
        AppEntry {
            id:       "delta",
            url:      "https://github.com/dandavison/delta",
            category: "git",
        },
        AppEntry {
            id:       "difft",
            url:      "https://github.com/Wilfred/difftastic",
            category: "git",
        },
        AppEntry {
            id:       "dockmate",
            url:      "https://github.com/shubh-io/DockMate",
            category: "containers",
        },
        AppEntry {
            id:       "dry",
            url:      "https://github.com/moncho/dry",
            category: "containers",
        },
        AppEntry {
            id:       "dust",
            url:      "https://github.com/bootandy/dust",
            category: "files",
        },
        AppEntry {
            id:       "eza",
            url:      "https://github.com/eza-community/eza",
            category: "files",
        },
        AppEntry {
            id:       "fd",
            url:      "https://github.com/sharkdp/fd",
            category: "files",
        },
        AppEntry {
            id:       "fnm",
            url:      "https://github.com/Schniz/fnm",
            category: "dev_envs",
        },
        AppEntry {
            id:       "fx",
            url:      "https://github.com/antonmedv/fx",
            category: "data",
        },
        AppEntry {
            id:       "fzf",
            url:      "https://github.com/junegunn/fzf",
            category: "shell",
        },
        AppEntry {
            id:       "gitleaks",
            url:      "https://github.com/gitleaks/gitleaks",
            category: "git",
        },
        AppEntry {
            id:       "gojq",
            url:      "https://github.com/itchyny/gojq",
            category: "data",
        },
        AppEntry {
            id:       "gonzo",
            url:      "https://github.com/control-theory/gonzo",
            category: "logs",
        },
        AppEntry {
            id:       "jid",
            url:      "https://github.com/simeji/jid",
            category: "data",
        },
        AppEntry {
            id:       "jq",
            url:      "https://github.com/jqlang/jq",
            category: "data",
        },
        AppEntry {
            id:       "jqp",
            url:      "https://github.com/noahgorstein/jqp",
            category: "data",
        },
        AppEntry {
            id:       "lazyjournal",
            url:      "https://github.com/Lifailon/lazyjournal",
            category: "logs",
        },
        AppEntry {
            id:       "lazydocker",
            url:      "https://github.com/jesseduffield/lazydocker",
            category: "containers",
        },
        AppEntry {
            id:       "lazygit",
            url:      "https://github.com/jesseduffield/lazygit",
            category: "git",
        },
        AppEntry {
            id:       "mdbook",
            url:      "https://github.com/rust-lang/mdBook",
            category: "dev_tools",
        },
        AppEntry {
            id:       "mergiraf",
            url:      "https://codeberg.org/mergiraf/mergiraf",
            category: "git",
        },
        AppEntry {
            id:       "mise",
            url:      "https://github.com/jdx/mise",
            category: "dev_envs",
        },
        AppEntry {
            id:       "neovide",
            url:      "https://github.com/neovide/neovide",
            category: "dev_tools",
        },
        AppEntry {
            id:       "rclone",
            url:      "https://github.com/rclone/rclone",
            category: "other",
        },
        AppEntry {
            id:       "restish",
            url:      "https://github.com/rest-sh/restish",
            category: "http",
        },
        AppEntry {
            id:       "rg",
            url:      "https://github.com/BurntSushi/ripgrep",
            category: "files",
        },
        AppEntry {
            id:       "rust-analyzer",
            url:      "https://github.com/rust-lang/rust-analyzer",
            category: "dev_tools",
        },
        AppEntry {
            id:       "sd",
            url:      "https://github.com/chmln/sd",
            category: "files",
        },
        AppEntry {
            id:       "sk",
            url:      "https://github.com/skim-rs/skim",
            category: "shell",
        },
        AppEntry {
            id:       "starship",
            url:      "https://github.com/starship/starship",
            category: "shell",
        },
        AppEntry {
            id:       "stylua",
            url:      "https://github.com/JohnnyMorganz/stylua",
            category: "dev_tools",
        },
        AppEntry {
            id:       "uv",
            url:      "https://github.com/astral-sh/uv",
            category: "dev_envs",
        },
        AppEntry {
            id:       "xh",
            url:      "https://github.com/ducaale/xh",
            category: "http",
        },
        AppEntry {
            id:       "xq",
            url:      "https://github.com/sibprogrammer/xq",
            category: "data",
        },
        AppEntry {
            id:       "yq",
            url:      "https://github.com/mikefarah/yq",
            category: "data",
        },
        AppEntry {
            id:       "zoxide",
            url:      "https://github.com/ajeetdsouza/zoxide",
            category: "shell",
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
        "rust-analyzer" => Some(Box::new(dev_tools::rust_analyzer::RustAnalyzer::new(client))),
        "sd" => Some(Box::new(files::sd_edit::SdEdit::new(client))),
        "sk" => Some(Box::new(shell::skim::Skim::new(client))),
        "starship" => Some(Box::new(shell::starship::Starship::new(client))),
        "stylua" => Some(Box::new(dev_tools::stylua::Stylua::new(client))),
        "uv" => Some(Box::new(dev_envs::uv::Uv::new(client))),
        "xh" => Some(Box::new(http::xh::Xh::new(client))),
        "xq" => Some(Box::new(data::xq::Xq::new(client))),
        "yq" => Some(Box::new(data::yq::Yq::new(client))),
        "zoxide" => Some(Box::new(shell::zoxide::Zoxide::new(client))),
        _ => None,
    }
}
