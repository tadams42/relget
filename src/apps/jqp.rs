use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::github::GithubClient;
use crate::types::{AppBinary, DownloadedAssets};
use crate::version::AppVersion;

pub struct Jqp {
    client: Arc<GithubClient>,
}

impl Jqp {
    const OWNER: &'static str = "noahgorstein";
    const REPO: &'static str = "jqp";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Jqp {
    fn exe_name(&self) -> &str { "jqp" }
    fn url(&self) -> &str { "https://github.com/noahgorstein/jqp" }

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
            .find(|a| a == "jqp_Linux_x86_64.tar.gz")
            .ok_or_else(|| anyhow!("Can't find jqp_Linux_x86_64.tar.gz"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;
        let exe = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "jqp")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find jqp in archive"))?;
        Ok(DownloadedAssets {
            binary: Some(AppBinary::new("jqp", extractor.extract(&exe)?)),
            ..Default::default()
        })
    }
}
