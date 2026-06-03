use anyhow::{Result, anyhow};
use std::sync::Arc;

use crate::apps::App;
use crate::clients::GithubClient;
use crate::installer::gen_completions_subcommand;
use crate::types::{AppBinary, Completion, AppAssets};
use crate::version::AppVersion;

pub struct Squix {
    client: Arc<GithubClient>,
}

impl Squix {
    pub const ID: &'static str = "squix";
    const OWNER: &'static str = "eduardofuncao";
    const REPO: &'static str = "squix";
    const EXE_NAME: &'static str = "squix";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Squix {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary:      Some(AppBinary::descriptor("squix")),
            completions: vec![Completion::zsh_desc("squix"), Completion::bash_desc("squix"), Completion::fish_desc("squix")],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release
            .asset_names()
            .into_iter()
            .find(|a| a.as_str() == "squix-linux-amd64")
            .ok_or_else(|| anyhow!("Can't find squix linux amd64 asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let completions = gen_completions_subcommand("squix", &asset.data, "completion")?;
        Ok(AppAssets {
            binary: Some(AppBinary::new("squix", asset.data)),
            completions,
            ..Default::default()
        })
    }
}
