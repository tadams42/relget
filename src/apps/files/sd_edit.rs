use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppAssets, AppBinary, ManPage};
use crate::version::AppVersion;

pub struct SdEdit {
    client: Arc<GithubClient>,
}

impl SdEdit {
    pub const ID: &'static str = "sd";
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

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::descriptor(Self::EXE_NAME)),
            man_pages: vec![ManPage::descriptor(1, "sd.1")],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release.find_asset(|a| {
            a.starts_with("sd-") && a.ends_with("-x86_64-unknown-linux-musl.tar.gz")
        })?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        Ok(AppAssets {
            binary: Some(AppBinary::new("sd", extractor.extract_by_filename("sd")?)),
            man_pages: vec![ManPage::new(
                1,
                "sd.1",
                extractor.extract_by_filename("sd.1")?,
            )],
            ..Default::default()
        })
    }
}
