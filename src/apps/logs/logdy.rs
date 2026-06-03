use anyhow::{Result, anyhow};
use std::sync::Arc;

use crate::apps::App;
use crate::clients::GithubClient;
use crate::installer::gen_completions_subcommand;
use crate::types::{AppBinary, Completion, AppAssets};
use crate::version::AppVersion;

pub struct Logdy {
    client: Arc<GithubClient>,
}

impl Logdy {
    pub const ID: &'static str = "logdy";
    pub const CATEGORY: &'static str = "logs";
    pub const DESCRIPTION: &'static str =
        "Web-based real-time log viewer with filtering and search";
    pub const URL: &'static str = "https://github.com/logdyhq/logdy-core";
    const OWNER: &'static str = "logdyhq";
    const REPO: &'static str = "logdy-core";
    const EXE_NAME: &'static str = "logdy";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Logdy {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary:      Some(AppBinary::descriptor("logdy")),
            completions: vec![Completion::zsh_desc("logdy"), Completion::bash_desc("logdy"), Completion::fish_desc("logdy")],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release
            .asset_names()
            .into_iter()
            .find(|a| a.as_str() == "logdy_linux_amd64")
            .ok_or_else(|| anyhow!("Can't find logdy linux amd64 asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let completions = gen_completions_subcommand("logdy", &asset.data, "completion")?;
        Ok(AppAssets {
            binary: Some(AppBinary::new("logdy", asset.data)),
            completions,
            ..Default::default()
        })
    }
}
