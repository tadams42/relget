use std::sync::Arc;

use crate::clients::{CodebergClient, GithubClient, GitlabClient};

use super::App;
use super::coding::{AstGrep, Grex, Neovide, Pyrefly, Rgx, Ruff, RustAnalyzer, Scc, Stylua, Ty};
use super::containers::{D4S, DockMate, Dry, LazyDocker};
use super::data_processing::{Dasel, Fx, GoJq, Jaq, Jid, Jq, Jqp, Qsv, QsvAll, Rsv, Xq, Yq};
use super::dev_envs::{Chezmoi, Fnm, Uv};
use super::docs_diag::{D2, Mdbook, Pdot, Pgplan, Tlrc};
use super::encryption::{Age, Doppler, Gocryptfs};
use super::files::{Bat, Eza, F2, FdFind, Rclone, Ripgrep, SdEdit, Trash, Yazi};
use super::git::{Delta, Difftastic, Gitleaks, Lazygit, Mergiraf, Worktrunk};
use super::http::{Caddy, Curlie, Hurl, Restish, Xh};
use super::logs::{Gonzo, LazyJournal, Logdy};
use super::networking::{Boring, Dog};
use super::shell::{Atuin, Carapace, Fzf, Skim, Starship, Zoxide};
use super::system::{Bottom, Btop, Dust, Dysk, Procs};

pub fn create_app(
    id: &str, gh_token: Option<String>, cb_token: Option<String>, gl_token: Option<String>,
    offline: bool,
) -> Option<Box<dyn App>> {
    let client = Arc::new(GithubClient::new(gh_token, offline));
    match id {
        Age::ID => Some(Box::new(Age::new(client))),
        Boring::ID => Some(Box::new(Boring::new(client))),
        Dog::ID => Some(Box::new(Dog::new(client))),
        AstGrep::ID => Some(Box::new(AstGrep::new(client))),
        Atuin::ID => Some(Box::new(Atuin::new(client))),
        Bat::ID => Some(Box::new(Bat::new(client))),
        Bottom::ID => Some(Box::new(Bottom::new(client))),
        Btop::ID => Some(Box::new(Btop::new(client))),
        Caddy::ID => Some(Box::new(Caddy::new(client))),
        Carapace::ID => Some(Box::new(Carapace::new(client))),
        Chezmoi::ID => Some(Box::new(Chezmoi::new(client))),
        Curlie::ID => Some(Box::new(Curlie::new(client))),
        D4S::ID => Some(Box::new(D4S::new(client))),
        Dasel::ID => Some(Box::new(Dasel::new(client))),
        Delta::ID => Some(Box::new(Delta::new(client))),
        Difftastic::ID => Some(Box::new(Difftastic::new(client))),
        DockMate::ID => Some(Box::new(DockMate::new(client))),
        Doppler::ID => Some(Box::new(Doppler::new(client))),
        Dry::ID => Some(Box::new(Dry::new(client))),
        Dust::ID => Some(Box::new(Dust::new(client))),
        Dysk::ID => Some(Box::new(Dysk::new(client))),
        Eza::ID => Some(Box::new(Eza::new(client))),
        F2::ID => Some(Box::new(F2::new(client))),
        FdFind::ID => Some(Box::new(FdFind::new(client))),
        Fnm::ID => Some(Box::new(Fnm::new(client))),
        Fx::ID => Some(Box::new(Fx::new(client))),
        Fzf::ID => Some(Box::new(Fzf::new(client))),
        Gitleaks::ID => Some(Box::new(Gitleaks::new(client))),
        Gocryptfs::ID => Some(Box::new(Gocryptfs::new(client))),
        GoJq::ID => Some(Box::new(GoJq::new(client))),
        Gonzo::ID => Some(Box::new(Gonzo::new(client))),
        Grex::ID => Some(Box::new(Grex::new(client))),
        Hurl::ID => Some(Box::new(Hurl::new(client))),
        Jaq::ID => Some(Box::new(Jaq::new(client))),
        Jid::ID => Some(Box::new(Jid::new(client))),
        Jq::ID => Some(Box::new(Jq::new(client))),
        Jqp::ID => Some(Box::new(Jqp::new(client))),
        LazyJournal::ID => Some(Box::new(LazyJournal::new(client))),
        LazyDocker::ID => Some(Box::new(LazyDocker::new(client))),
        D2::ID => Some(Box::new(D2::new(client))),
        Lazygit::ID => Some(Box::new(Lazygit::new(client))),
        Logdy::ID => Some(Box::new(Logdy::new(client))),
        Mdbook::ID => Some(Box::new(Mdbook::new(client))),
        Mergiraf::ID => {
            Some(Box::new(Mergiraf::new(Arc::new(CodebergClient::new(
                cb_token, offline,
            )))))
        }
        Pdot::ID => Some(Box::new(Pdot::new(Arc::new(GitlabClient::new(gl_token, offline))))),
        Neovide::ID => Some(Box::new(Neovide::new(client))),
        Pgplan::ID => Some(Box::new(Pgplan::new(client))),
        Procs::ID => Some(Box::new(Procs::new(client))),
        Pyrefly::ID => Some(Box::new(Pyrefly::new(client))),
        Rclone::ID => Some(Box::new(Rclone::new(client))),
        Restish::ID => Some(Box::new(Restish::new(client))),
        Rgx::ID => Some(Box::new(Rgx::new(client))),
        Ruff::ID => Some(Box::new(Ruff::new(client))),
        Ripgrep::ID => Some(Box::new(Ripgrep::new(client))),
        Qsv::ID => Some(Box::new(Qsv::new(client))),
        QsvAll::ID => Some(Box::new(QsvAll::new(client))),
        Rsv::ID => Some(Box::new(Rsv::new(client))),
        RustAnalyzer::ID => Some(Box::new(RustAnalyzer::new(client))),
        Scc::ID => Some(Box::new(Scc::new(client))),
        SdEdit::ID => Some(Box::new(SdEdit::new(client))),
        Skim::ID => Some(Box::new(Skim::new(client))),
        Starship::ID => Some(Box::new(Starship::new(client))),
        Stylua::ID => Some(Box::new(Stylua::new(client))),
        Tlrc::ID => Some(Box::new(Tlrc::new(client))),
        Ty::ID => Some(Box::new(Ty::new(client))),
        Trash::ID => Some(Box::new(Trash::new(client))),
        Uv::ID => Some(Box::new(Uv::new(client))),
        Xh::ID => Some(Box::new(Xh::new(client))),
        Xq::ID => Some(Box::new(Xq::new(client))),
        Yazi::ID => Some(Box::new(Yazi::new(client))),
        Worktrunk::ID => Some(Box::new(Worktrunk::new(client))),
        Yq::ID => Some(Box::new(Yq::new(client))),
        Zoxide::ID => Some(Box::new(Zoxide::new(client))),
        _ => None,
    }
}
