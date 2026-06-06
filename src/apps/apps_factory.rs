use std::sync::Arc;

use crate::clients::{CodebergClient, GithubClient, GitlabClient};

use super::App;
use super::coding::{AstGrep, Fend, Grex, Hyperfine, Neovide, Pyrefly, Replibyte, Rgx, Ruff, RustAnalyzer, Scc, Sqruff, Sttr, Stylua, Ty};
use super::containers::{Ctop, D4S, DockMate, Dry, LazyDocker};
use super::data_processing::{Dasel, Fq, Fx, GoJq, Jaq, Jd, Jid, Jiq, Jq, Jqp, JsonGrep, Miller, Qq, Qsv, QsvAll, Rsv, Tabiew, Taplo, Tv, Xan, Xq, Yq};
use super::dev_envs::{Chezmoi, Fnm, Uv};
use super::docs_diag::{Agg, Asciinema, D2, Mdbook, Pdot, Pgplan, Tbls, Tlrc};
use super::encryption::{Age, Doppler, Gocryptfs, Rage};
use super::files::{Bat, Choose, Eza, F2, FdFind, Rclone, Ripgrep, SdEdit, Termscp, Trash, Xplr, Yazi};
use super::git::{Delta, Difftastic, Gitleaks, Lazygit, Mergiraf, Worktrunk};
use super::http::{Caddy, Curlie, Hurl, Restish, Xh};
use super::logs::{Dtop, Gonzo, Hl, LazyJournal, Logdy, Loggo, Nerdlog, Rhit, Tailspin};
use super::networking::{Boring, Dog, Doggo};
use super::shell::{Atuin, Carapace, Fzf, Skim, Starship, Zoxide};
use super::system::{Bottom, Btop, Duf, Dust, Dysk, Erdtree, Procs};

pub fn create_app(
    id: &str, gh_token: Option<String>, cb_token: Option<String>, gl_token: Option<String>,
    offline: bool,
) -> Option<Box<dyn App>> {
    let client = Arc::new(GithubClient::new(gh_token, offline));
    match id {
        Age::ID => Some(Box::new(Age::new(client))),
        Agg::ID => Some(Box::new(Agg::new(client))),
        Asciinema::ID => Some(Box::new(Asciinema::new(client))),
        Boring::ID => Some(Box::new(Boring::new(client))),
        Dog::ID => Some(Box::new(Dog::new(client))),
        Doggo::ID => Some(Box::new(Doggo::new(client))),
        AstGrep::ID => Some(Box::new(AstGrep::new(client))),
        Atuin::ID => Some(Box::new(Atuin::new(client))),
        Bat::ID => Some(Box::new(Bat::new(client))),
        Bottom::ID => Some(Box::new(Bottom::new(client))),
        Btop::ID => Some(Box::new(Btop::new(client))),
        Caddy::ID => Some(Box::new(Caddy::new(client))),
        Carapace::ID => Some(Box::new(Carapace::new(client))),
        Choose::ID => Some(Box::new(Choose::new(client))),
        Chezmoi::ID => Some(Box::new(Chezmoi::new(client))),
        Curlie::ID => Some(Box::new(Curlie::new(client))),
        Ctop::ID => Some(Box::new(Ctop::new(client))),
        D4S::ID => Some(Box::new(D4S::new(client))),
        Dasel::ID => Some(Box::new(Dasel::new(client))),
        Dtop::ID => Some(Box::new(Dtop::new(client))),
        Delta::ID => Some(Box::new(Delta::new(client))),
        Difftastic::ID => Some(Box::new(Difftastic::new(client))),
        DockMate::ID => Some(Box::new(DockMate::new(client))),
        Doppler::ID => Some(Box::new(Doppler::new(client))),
        Dry::ID => Some(Box::new(Dry::new(client))),
        Duf::ID => Some(Box::new(Duf::new(client))),
        Erdtree::ID => Some(Box::new(Erdtree::new(client))),
        Dust::ID => Some(Box::new(Dust::new(client))),
        Dysk::ID => Some(Box::new(Dysk::new(client))),
        Eza::ID => Some(Box::new(Eza::new(client))),
        F2::ID => Some(Box::new(F2::new(client))),
        FdFind::ID => Some(Box::new(FdFind::new(client))),
        Fend::ID => Some(Box::new(Fend::new(client))),
        Fnm::ID => Some(Box::new(Fnm::new(client))),
        Fq::ID => Some(Box::new(Fq::new(client))),
        Fx::ID => Some(Box::new(Fx::new(client))),
        Fzf::ID => Some(Box::new(Fzf::new(client))),
        Gitleaks::ID => Some(Box::new(Gitleaks::new(client))),
        Gocryptfs::ID => Some(Box::new(Gocryptfs::new(client))),
        GoJq::ID => Some(Box::new(GoJq::new(client))),
        Gonzo::ID => Some(Box::new(Gonzo::new(client))),
        Hl::ID => Some(Box::new(Hl::new(client))),
        Grex::ID => Some(Box::new(Grex::new(client))),
        Hurl::ID => Some(Box::new(Hurl::new(client))),
        Hyperfine::ID => Some(Box::new(Hyperfine::new(client))),
        Jaq::ID => Some(Box::new(Jaq::new(client))),
        Jd::ID => Some(Box::new(Jd::new(client))),
        Jid::ID => Some(Box::new(Jid::new(client))),
        Jiq::ID => Some(Box::new(Jiq::new(client))),
        Jq::ID => Some(Box::new(Jq::new(client))),
        JsonGrep::ID => Some(Box::new(JsonGrep::new(client))),
        Jqp::ID => Some(Box::new(Jqp::new(client))),
        LazyJournal::ID => Some(Box::new(LazyJournal::new(client))),
        LazyDocker::ID => Some(Box::new(LazyDocker::new(client))),
        D2::ID => Some(Box::new(D2::new(client))),
        Lazygit::ID => Some(Box::new(Lazygit::new(client))),
        Logdy::ID => Some(Box::new(Logdy::new(client))),
        Loggo::ID => Some(Box::new(Loggo::new(client))),
        Mdbook::ID => Some(Box::new(Mdbook::new(client))),
        Miller::ID => Some(Box::new(Miller::new(client))),
        Mergiraf::ID => {
            Some(Box::new(Mergiraf::new(Arc::new(CodebergClient::new(
                cb_token, offline,
            )))))
        }
        Pdot::ID => Some(Box::new(Pdot::new(Arc::new(GitlabClient::new(gl_token, offline))))),
        Neovide::ID => Some(Box::new(Neovide::new(client))),
        Nerdlog::ID => Some(Box::new(Nerdlog::new(client))),
        Pgplan::ID => Some(Box::new(Pgplan::new(client))),
        Procs::ID => Some(Box::new(Procs::new(client))),
        Pyrefly::ID => Some(Box::new(Pyrefly::new(client))),
        Rage::ID => Some(Box::new(Rage::new(client))),
        Rhit::ID => Some(Box::new(Rhit::new(client))),
        Rclone::ID => Some(Box::new(Rclone::new(client))),
        Restish::ID => Some(Box::new(Restish::new(client))),
        Replibyte::ID => Some(Box::new(Replibyte::new(client))),
        Rgx::ID => Some(Box::new(Rgx::new(client))),
        Ruff::ID => Some(Box::new(Ruff::new(client))),
        Ripgrep::ID => Some(Box::new(Ripgrep::new(client))),
        Qq::ID => Some(Box::new(Qq::new(client))),
        Qsv::ID => Some(Box::new(Qsv::new(client))),
        QsvAll::ID => Some(Box::new(QsvAll::new(client))),
        Rsv::ID => Some(Box::new(Rsv::new(client))),
        RustAnalyzer::ID => Some(Box::new(RustAnalyzer::new(client))),
        Scc::ID => Some(Box::new(Scc::new(client))),
        Sqruff::ID => Some(Box::new(Sqruff::new(client))),
        SdEdit::ID => Some(Box::new(SdEdit::new(client))),
        Skim::ID => Some(Box::new(Skim::new(client))),
        Starship::ID => Some(Box::new(Starship::new(client))),
        Sttr::ID => Some(Box::new(Sttr::new(client))),
        Stylua::ID => Some(Box::new(Stylua::new(client))),
        Tabiew::ID => Some(Box::new(Tabiew::new(client))),
        Tailspin::ID => Some(Box::new(Tailspin::new(client))),
        Taplo::ID => Some(Box::new(Taplo::new(client))),
        Tbls::ID => Some(Box::new(Tbls::new(client))),
        Termscp::ID => Some(Box::new(Termscp::new(client))),
        Tlrc::ID => Some(Box::new(Tlrc::new(client))),
        Tv::ID => Some(Box::new(Tv::new(client))),
        Ty::ID => Some(Box::new(Ty::new(client))),
        Trash::ID => Some(Box::new(Trash::new(client))),
        Uv::ID => Some(Box::new(Uv::new(client))),
        Xh::ID => Some(Box::new(Xh::new(client))),
        Xan::ID => Some(Box::new(Xan::new(client))),
        Xq::ID => Some(Box::new(Xq::new(client))),
        Xplr::ID => Some(Box::new(Xplr::new(client))),
        Yazi::ID => Some(Box::new(Yazi::new(client))),
        Worktrunk::ID => Some(Box::new(Worktrunk::new(client))),
        Yq::ID => Some(Box::new(Yq::new(client))),
        Zoxide::ID => Some(Box::new(Zoxide::new(client))),
        _ => None,
    }
}
