use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::github::GithubClient;
use crate::types::{AppBinary, DownloadedAssets};
use crate::version::AppVersion;

pub struct D4S {
    client: Arc<GithubClient>,
}

impl D4S {
    const OWNER: &'static str = "jr-k";
    const REPO: &'static str = "d4s";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for D4S {
    fn exe_name(&self) -> &str { "d4s" }
    fn url(&self) -> &str { "https://github.com/jr-k/d4s" }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn parse_installed_version(&self, data: &str) -> Option<AppVersion> {
        // Look for a line containing "Version" and grab the last word
        let line = data.lines().find(|l| l.contains("Version"))?;
        let ver = line.split_whitespace().last()?;
        AppVersion::parse(ver)
    }

    fn download(&self) -> Result<DownloadedAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release
            .asset_names()
            .into_iter()
            .find(|a| a.starts_with("d4s") && a.ends_with("linux_amd64.tar.gz"))
            .ok_or_else(|| anyhow!("Can't find d4s asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;
        let exe = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "d4s")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find d4s in archive"))?;
        Ok(DownloadedAssets {
            binary: Some(AppBinary::new("d4s", extractor.extract(&exe)?)),
            ..Default::default()
        })
    }
}
