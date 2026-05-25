use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::github::GithubClient;
use crate::types::{AppBinary, DownloadedAssets};
use crate::version::AppVersion;

pub struct Stylua {
    client: Arc<GithubClient>,
}

impl Stylua {
    pub const DESCRIPTION: &'static str = "Opinionated Lua code formatter";
    pub const URL: &'static str = "https://github.com/JohnnyMorganz/stylua";
    const OWNER: &'static str = "JohnnyMorganz";
    const REPO: &'static str = "stylua";
    const EXE_NAME: &'static str = "stylua";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Stylua {
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
            .find(|a| a == "stylua-linux-x86_64.zip")
            .ok_or_else(|| anyhow!("Can't find stylua-linux-x86_64.zip"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;
        let exe = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "stylua")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find stylua in archive"))?;
        Ok(DownloadedAssets {
            binary: Some(AppBinary::new("stylua", extractor.extract(&exe)?)),
            ..Default::default()
        })
    }
}
