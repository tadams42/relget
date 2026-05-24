use anyhow::{Result, anyhow};
use std::sync::Arc;

use crate::apps::App;
use crate::github::GithubClient;
use crate::types::{AppBinary, DownloadedAssets};
use crate::version::AppVersion;

pub struct Fx {
    client: Arc<GithubClient>,
}

impl Fx {
    pub const DESCRIPTION: &'static str = "Terminal JSON viewer and interactive processor";
    pub const URL: &'static str = "https://github.com/antonmedv/fx";
    const OWNER: &'static str = "antonmedv";
    const REPO: &'static str = "fx";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Fx {
    fn exe_name(&self) -> &str { "fx" }

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
            .find(|a| a == "fx_linux_amd64")
            .ok_or_else(|| anyhow!("Can't find fx_linux_amd64 asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        Ok(DownloadedAssets {
            binary: Some(AppBinary::new("fx", asset.data)),
            ..Default::default()
        })
    }
}
