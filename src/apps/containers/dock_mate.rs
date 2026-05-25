use anyhow::{Result, anyhow};
use std::sync::Arc;

use crate::apps::App;
use crate::github::GithubClient;
use crate::types::{AppBinary, DownloadedAssets};
use crate::version::AppVersion;

pub struct DockMate {
    client: Arc<GithubClient>,
}

impl DockMate {
    pub const DESCRIPTION: &'static str = "Terminal UI for managing Docker containers and images";
    pub const URL: &'static str = "https://github.com/shubh-io/DockMate";
    const OWNER: &'static str = "shubh-io";
    const REPO: &'static str = "DockMate";
    const EXE_NAME: &'static str = "dockmate";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for DockMate {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn download(&self) -> Result<DownloadedAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release
            .asset_names()
            .into_iter()
            .find(|a| a == "dockmate-linux-amd64")
            .ok_or_else(|| anyhow!("Can't find dockmate-linux-amd64 asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        Ok(DownloadedAssets {
            binary: Some(AppBinary::new("dockmate", asset.data)),
            ..Default::default()
        })
    }
}
