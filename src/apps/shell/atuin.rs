use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::installer::gen_completions_shell_flag;
use crate::types::{AppBinary, Completion, AppAssets};
use crate::version::AppVersion;

pub struct Atuin {
    client: Arc<GithubClient>,
}

impl Atuin {
    pub const ID: &'static str = "atuin";
    pub const CATEGORY: &'static str = "shell";
    pub const DESCRIPTION: &'static str = "Shell history search backed by SQLite with sync";
    pub const URL: &'static str = "https://github.com/atuinsh/atuin";
    const OWNER: &'static str = "atuinsh";
    const REPO: &'static str = "atuin";
    const EXE_NAME: &'static str = "atuin";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Atuin {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary:      Some(AppBinary::descriptor("atuin")),
            completions: vec![Completion::zsh_desc("atuin"), Completion::bash_desc("atuin"), Completion::fish_desc("atuin")],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release
            .asset_names()
            .into_iter()
            .find(|a| a == "atuin-x86_64-unknown-linux-gnu.tar.gz")
            .ok_or_else(|| anyhow!("Can't find atuin asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;
        let exe = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "atuin")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find atuin in archive"))?;
        let binary_data = extractor.extract(&exe)?;
        let completions =
            gen_completions_shell_flag("atuin", &binary_data, "gen-completions", "--shell")?;
        Ok(AppAssets {
            binary: Some(AppBinary::new("atuin", binary_data)),
            completions,
            ..Default::default()
        })
    }
}
