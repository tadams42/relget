use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::github::GithubClient;
use crate::types::{AppBinary, DownloadedAssets, ManPage};
use crate::version::AppVersion;

pub struct Jq {
    client: Arc<GithubClient>,
}

impl Jq {
    const OWNER: &'static str = "jqlang";
    const REPO: &'static str = "jq";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Jq {
    fn exe_name(&self) -> &str { "jq" }
    fn url(&self) -> &str { "https://github.com/jqlang/jq" }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn download(&self) -> Result<DownloadedAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;

        // Man page from source tarball
        let tar_name = release
            .asset_names()
            .into_iter()
            .find(|a| a.starts_with("jq-") && a.ends_with(".tar.gz"))
            .ok_or_else(|| anyhow!("Can't find jq source tarball"))?;
        let tar_asset = self
            .client
            .download_asset(Self::OWNER, Self::REPO, &tar_name)?;
        let extractor = ArchiveExtractor::new(&tar_name, tar_asset.data);
        let members = extractor.members()?;
        let man = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "jq.1")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find jq.1 in source tarball"))?;
        let man_data = extractor.extract(&man)?;

        // Binary (raw file, not an archive)
        let bin_name = release
            .asset_names()
            .into_iter()
            .find(|a| a == "jq-linux-amd64")
            .ok_or_else(|| anyhow!("Can't find jq-linux-amd64 binary asset"))?;
        let bin_asset = self
            .client
            .download_asset(Self::OWNER, Self::REPO, &bin_name)?;

        Ok(DownloadedAssets {
            binary: Some(AppBinary::new("jq", bin_asset.data)),
            man_pages: vec![ManPage::new(1, "jq.1", man_data)],
            ..Default::default()
        })
    }
}
