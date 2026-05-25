use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::cache::GhRelease;
use crate::github::GithubClient;
use crate::types::{AppBinary, DownloadedAssets};
use crate::version::AppVersion;

pub const OWNER: &str = "dathere";
pub const REPO: &str = "qsv";

pub fn gnu_zip_asset_name(release: &GhRelease) -> Result<String> {
    release
        .asset_names()
        .into_iter()
        .find(|a| a.contains("x86_64-unknown-linux-gnu") && a.ends_with(".zip"))
        .ok_or_else(|| anyhow!("Can't find qsv x86_64-unknown-linux-gnu zip asset"))
}

pub fn extract_named(
    extractor: &ArchiveExtractor, members: &[String], name: &str,
) -> Result<Vec<u8>> {
    let entry = members
        .iter()
        .find(|m| Path::new(m).file_name().map(|f| f == name).unwrap_or(false))
        .cloned()
        .ok_or_else(|| anyhow!("Can't find {} in archive", name))?;
    extractor.extract(&entry)
}

pub struct Qsv {
    client: Arc<GithubClient>,
}

impl Qsv {
    pub const DESCRIPTION: &'static str = "High-performance CSV data-wrangling toolkit";
    pub const URL: &'static str = "https://github.com/dathere/qsv";
    const EXE_NAME: &'static str = "qsv";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Qsv {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client.latest_release(OWNER, REPO)?.version()
    }

    fn download(&self) -> Result<DownloadedAssets> {
        let release = self.client.latest_release(OWNER, REPO)?;
        let name = gnu_zip_asset_name(&release)?;
        let asset = self.client.download_asset(OWNER, REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;
        let data = extract_named(&extractor, &members, "qsv")?;
        Ok(DownloadedAssets {
            binary: Some(AppBinary::new("qsv", data)),
            ..Default::default()
        })
    }
}
