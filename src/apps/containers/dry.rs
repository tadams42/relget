use anyhow::{Result, anyhow};
use std::sync::Arc;

use crate::apps::App;
use crate::clients::GithubClient;
use crate::types::{AppBinary, AppAssets};
use crate::version::AppVersion;

pub struct Dry {
    client: Arc<GithubClient>,
}

impl Dry {
    pub const ID: &'static str = "dry";
    pub const CATEGORY: &'static str = "containers";
    pub const DESCRIPTION: &'static str = "Interactive terminal application for Docker management";
    pub const URL: &'static str = "https://github.com/moncho/dry";
    const OWNER: &'static str = "moncho";
    const REPO: &'static str = "dry";
    const EXE_NAME: &'static str = "dry";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Dry {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary:      Some(AppBinary::descriptor("dry")),
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release
            .asset_names()
            .into_iter()
            .find(|a| a == "dry-linux-amd64")
            .ok_or_else(|| anyhow!("Can't find dry-linux-amd64 asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        Ok(AppAssets {
            binary: Some(AppBinary::new("dry", asset.data)),
            ..Default::default()
        })
    }
}
