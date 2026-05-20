use anyhow::{Result, anyhow};
use std::sync::Arc;

use crate::apps::App;
use crate::github::GithubClient;
use crate::types::{AppBinary, DownloadedAssets};
use crate::version::AppVersion;

pub struct Dry {
    client: Arc<GithubClient>,
}

impl Dry {
    const OWNER: &'static str = "moncho";
    const REPO: &'static str = "dry";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Dry {
    fn exe_name(&self) -> &str { "dry" }
    fn url(&self) -> &str { "https://github.com/moncho/dry" }
    fn installed_version_word_index(&self) -> isize { 2 }

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
            .find(|a| a == "dry-linux-amd64")
            .ok_or_else(|| anyhow!("Can't find dry-linux-amd64 asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        Ok(DownloadedAssets {
            binary: Some(AppBinary::new("dry", asset.data)),
            ..Default::default()
        })
    }
}
