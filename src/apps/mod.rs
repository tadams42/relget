use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::installer::install_assets;
use crate::types::DownloadedAssets;
use crate::version::AppVersion;

pub mod aqua;
pub mod ast_grep;
pub mod atuin;
pub mod bat;
pub mod caddy;
pub mod carapace;
pub mod chezmoi;
pub mod d4s;
pub mod dasel;
pub mod delta;
pub mod difftastic;
pub mod dock_mate;
pub mod dry;
pub mod eza;
pub mod fd_find;
pub mod fnm;
pub mod fx;
pub mod fzf;
pub mod gitleaks;
pub mod go;
pub mod gojq;
pub mod gonzo;
pub mod jid;
pub mod jq;
pub mod jqp;
pub mod lazy_journal;
pub mod lazydocker;
pub mod lazygit;
pub mod mdbook;
pub mod mergiraf;
pub mod mise;
pub mod neovide;
pub mod rclone;
pub mod restish;
pub mod ripgrep;
pub mod rust_analyzer;
pub mod sd_edit;
pub mod skim;
pub mod starship;
pub mod stylua;
pub mod uv;
pub mod xq;
pub mod yq;
pub mod zoxide;

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
    pub id:  &'static str,
    pub url: &'static str,
}

pub fn all_app_entries() -> Vec<AppEntry> {
    vec![
        AppEntry {
            id:  "aqua",
            url: "https://github.com/aquaproj/aqua",
        },
        AppEntry {
            id:  "ast-grep",
            url: "https://github.com/ast-grep/ast-grep",
        },
        AppEntry {
            id:  "atuin",
            url: "https://github.com/atuinsh/atuin",
        },
        AppEntry {
            id:  "bat",
            url: "https://github.com/sharkdp/bat",
        },
        AppEntry {
            id:  "caddy",
            url: "https://github.com/caddyserver/caddy",
        },
        AppEntry {
            id:  "carapace",
            url: "https://github.com/carapace-sh/carapace-bin",
        },
        AppEntry {
            id:  "chezmoi",
            url: "https://github.com/twpayne/chezmoi",
        },
        AppEntry {
            id:  "d4s",
            url: "https://github.com/jr-k/d4s",
        },
        AppEntry {
            id:  "dasel",
            url: "https://github.com/TomWright/dasel",
        },
        AppEntry {
            id:  "delta",
            url: "https://github.com/dandavison/delta",
        },
        AppEntry {
            id:  "difft",
            url: "https://github.com/Wilfred/difftastic",
        },
        AppEntry {
            id:  "dockmate",
            url: "https://github.com/shubh-io/DockMate",
        },
        AppEntry {
            id:  "dry",
            url: "https://github.com/moncho/dry",
        },
        AppEntry {
            id:  "eza",
            url: "https://github.com/eza-community/eza",
        },
        AppEntry {
            id:  "fd",
            url: "https://github.com/sharkdp/fd",
        },
        AppEntry {
            id:  "fnm",
            url: "https://github.com/Schniz/fnm",
        },
        AppEntry {
            id:  "fx",
            url: "https://github.com/antonmedv/fx",
        },
        AppEntry {
            id:  "fzf",
            url: "https://github.com/junegunn/fzf",
        },
        AppEntry {
            id:  "gitleaks",
            url: "https://github.com/gitleaks/gitleaks",
        },
        AppEntry {
            id:  "go",
            url: "https://go.dev/",
        },
        AppEntry {
            id:  "gojq",
            url: "https://github.com/itchyny/gojq",
        },
        AppEntry {
            id:  "gonzo",
            url: "https://github.com/control-theory/gonzo",
        },
        AppEntry {
            id:  "jid",
            url: "https://github.com/simeji/jid",
        },
        AppEntry {
            id:  "jq",
            url: "https://github.com/jqlang/jq",
        },
        AppEntry {
            id:  "jqp",
            url: "https://github.com/noahgorstein/jqp",
        },
        AppEntry {
            id:  "lazyjournal",
            url: "https://github.com/Lifailon/lazyjournal",
        },
        AppEntry {
            id:  "lazydocker",
            url: "https://github.com/jesseduffield/lazydocker",
        },
        AppEntry {
            id:  "lazygit",
            url: "https://github.com/jesseduffield/lazygit",
        },
        AppEntry {
            id:  "mdbook",
            url: "https://github.com/rust-lang/mdBook",
        },
        AppEntry {
            id:  "mergiraf",
            url: "https://codeberg.org/mergiraf/mergiraf",
        },
        AppEntry {
            id:  "mise",
            url: "https://github.com/jdx/mise",
        },
        AppEntry {
            id:  "neovide",
            url: "https://github.com/neovide/neovide",
        },
        AppEntry {
            id:  "rclone",
            url: "https://github.com/rclone/rclone",
        },
        AppEntry {
            id:  "restish",
            url: "https://github.com/rest-sh/restish",
        },
        AppEntry {
            id:  "rg",
            url: "https://github.com/BurntSushi/ripgrep",
        },
        AppEntry {
            id:  "rust-analyzer",
            url: "https://github.com/rust-lang/rust-analyzer",
        },
        AppEntry {
            id:  "sd",
            url: "https://github.com/chmln/sd",
        },
        AppEntry {
            id:  "sk",
            url: "https://github.com/skim-rs/skim",
        },
        AppEntry {
            id:  "starship",
            url: "https://github.com/starship/starship",
        },
        AppEntry {
            id:  "stylua",
            url: "https://github.com/JohnnyMorganz/stylua",
        },
        AppEntry {
            id:  "uv",
            url: "https://github.com/astral-sh/uv",
        },
        AppEntry {
            id:  "xq",
            url: "https://github.com/sibprogrammer/xq",
        },
        AppEntry {
            id:  "yq",
            url: "https://github.com/mikefarah/yq",
        },
        AppEntry {
            id:  "zoxide",
            url: "https://github.com/ajeetdsouza/zoxide",
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
        "aqua" => Some(Box::new(aqua::Aqua::new(client))),
        "ast-grep" => Some(Box::new(ast_grep::AstGrep::new(client))),
        "atuin" => Some(Box::new(atuin::Atuin::new(client))),
        "bat" => Some(Box::new(bat::Bat::new(client))),
        "caddy" => Some(Box::new(caddy::Caddy::new(client))),
        "carapace" => Some(Box::new(carapace::Carapace::new(client))),
        "chezmoi" => Some(Box::new(chezmoi::Chezmoi::new(client))),
        "d4s" => Some(Box::new(d4s::D4S::new(client))),
        "dasel" => Some(Box::new(dasel::Dasel::new(client))),
        "delta" => Some(Box::new(delta::Delta::new(client))),
        "difft" => Some(Box::new(difftastic::Difftastic::new(client))),
        "dockmate" => Some(Box::new(dock_mate::DockMate::new(client))),
        "dry" => Some(Box::new(dry::Dry::new(client))),
        "eza" => Some(Box::new(eza::Eza::new(client))),
        "fd" => Some(Box::new(fd_find::FdFind::new(client))),
        "fnm" => Some(Box::new(fnm::Fnm::new(client))),
        "fx" => Some(Box::new(fx::Fx::new(client))),
        "fzf" => Some(Box::new(fzf::Fzf::new(client))),
        "gitleaks" => Some(Box::new(gitleaks::Gitleaks::new(client))),
        "go" => Some(Box::new(go::Go::new())),
        "gojq" => Some(Box::new(gojq::GoJq::new(client))),
        "gonzo" => Some(Box::new(gonzo::Gonzo::new(client))),
        "jid" => Some(Box::new(jid::Jid::new(client))),
        "jq" => Some(Box::new(jq::Jq::new(client))),
        "jqp" => Some(Box::new(jqp::Jqp::new(client))),
        "lazyjournal" => Some(Box::new(lazy_journal::LazyJournal::new(client))),
        "lazydocker" => Some(Box::new(lazydocker::LazyDocker::new(client))),
        "lazygit" => Some(Box::new(lazygit::Lazygit::new(client))),
        "mdbook" => Some(Box::new(mdbook::Mdbook::new(client))),
        "mergiraf" => {
            Some(Box::new(mergiraf::Mergiraf::new(Arc::new(CodebergClient::new(
                cb_token, offline,
            )))))
        }
        "mise" => Some(Box::new(mise::Mise::new(client))),
        "neovide" => Some(Box::new(neovide::Neovide::new(client))),
        "rclone" => Some(Box::new(rclone::Rclone::new(client))),
        "restish" => Some(Box::new(restish::Restish::new(client))),
        "rg" => Some(Box::new(ripgrep::Ripgrep::new(client))),
        "rust-analyzer" => Some(Box::new(rust_analyzer::RustAnalyzer::new(client))),
        "sd" => Some(Box::new(sd_edit::SdEdit::new(client))),
        "sk" => Some(Box::new(skim::Skim::new(client))),
        "starship" => Some(Box::new(starship::Starship::new(client))),
        "stylua" => Some(Box::new(stylua::Stylua::new(client))),
        "uv" => Some(Box::new(uv::Uv::new(client))),
        "xq" => Some(Box::new(xq::Xq::new(client))),
        "yq" => Some(Box::new(yq::Yq::new(client))),
        "zoxide" => Some(Box::new(zoxide::Zoxide::new(client))),
        _ => None,
    }
}
