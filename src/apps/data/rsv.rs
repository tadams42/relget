use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::github::GithubClient;
use crate::types::{AppBinary, DownloadedAssets};
use crate::version::AppVersion;

pub struct Rsv {
    client: Arc<GithubClient>,
}

impl Rsv {
    const OWNER: &'static str = "ribbondz";
    const REPO: &'static str = "rsv";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Rsv {
    fn exe_name(&self) -> &str { "rsv" }
    fn url(&self) -> &str { "https://github.com/ribbondz/rsv" }

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
            .find(|a| a == "x86_64-unknown-linux-musl.zip")
            .ok_or_else(|| anyhow!("Can't find x86_64-unknown-linux-musl.zip"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;
        let exe = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "rsv")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find rsv in archive"))?;
        Ok(DownloadedAssets {
            binary: Some(AppBinary::new("rsv", extractor.extract(&exe)?)),
            ..Default::default()
        })
    }
}
