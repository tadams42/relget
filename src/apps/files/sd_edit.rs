use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::github::GithubClient;
use crate::types::{AppBinary, DownloadedAssets, ManPage};
use crate::version::AppVersion;

pub struct SdEdit {
    client: Arc<GithubClient>,
}

impl SdEdit {
    pub const DESCRIPTION: &'static str = "Intuitive find-and-replace command, a sed alternative";
    pub const URL: &'static str = "https://github.com/chmln/sd";
    const OWNER: &'static str = "chmln";
    const REPO: &'static str = "sd";
    const EXE_NAME: &'static str = "sd";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for SdEdit {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    // NOTE: sd v1.1.0 has an upstream packaging bug — the binary inside the release
    // tarball reports "sd 1.0.0" regardless of the actual release tag. This causes
    // relget to reinstall sd on every run until upstream fixes their release pipeline.

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
            .find(|a| a.starts_with("sd-") && a.ends_with("-x86_64-unknown-linux-musl.tar.gz"))
            .ok_or_else(|| anyhow!("Can't find sd asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;

        let exe = members
            .iter()
            .find(|m| Path::new(m).file_name().map(|f| f == "sd").unwrap_or(false))
            .cloned()
            .ok_or_else(|| anyhow!("Can't find sd in archive"))?;
        let man = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "sd.1")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find sd.1 in archive"))?;

        Ok(DownloadedAssets {
            binary: Some(AppBinary::new("sd", extractor.extract(&exe)?)),
            man_pages: vec![ManPage::new(1, "sd.1", extractor.extract(&man)?)],
            ..Default::default()
        })
    }
}
