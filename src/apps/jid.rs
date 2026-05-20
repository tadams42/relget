use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::github::GithubClient;
use crate::types::{AppBinary, DownloadedAssets};
use crate::version::AppVersion;

pub struct Jid {
    client: Arc<GithubClient>,
}

impl Jid {
    const OWNER: &'static str = "simeji";
    const REPO: &'static str = "jid";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Jid {
    fn exe_name(&self) -> &str { "jid" }
    fn url(&self) -> &str { "https://github.com/simeji/jid" }

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
            .find(|a| a == "jid_linux_amd64.zip")
            .ok_or_else(|| anyhow!("Can't find jid_linux_amd64.zip"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;
        let exe = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "jid")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find jid in archive"))?;
        Ok(DownloadedAssets {
            binary: Some(AppBinary::new("jid", extractor.extract(&exe)?)),
            ..Default::default()
        })
    }
}
