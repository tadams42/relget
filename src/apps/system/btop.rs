use std::sync::Arc;

use anyhow::{Result, anyhow};

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppAssets, AppBinary};
use crate::version::AppVersion;

pub struct Btop {
    client: Arc<GithubClient>,
}

impl Btop {
    pub const ID: &'static str = "btop";
    const OWNER: &'static str = "aristocratos";
    const REPO: &'static str = "btop";
    const EXE_NAME: &'static str = "btop";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Btop {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::descriptor(Self::EXE_NAME)),
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let bin_name = release
            .asset_names()
            .into_iter()
            .find(|a| a.contains("x86_64-unknown-linux-musl") && a.ends_with(".tar.gz"))
            .ok_or_else(|| anyhow!("Can't find btop musl binary asset"))?;
        let asset = self
            .client
            .download_asset(Self::OWNER, Self::REPO, &bin_name)?;
        let extractor = ArchiveExtractor::new(&bin_name, asset.data);
        let binary_data = extractor.extract_by_filename(Self::EXE_NAME)?;
        Ok(AppAssets {
            binary: Some(AppBinary::new(Self::EXE_NAME, binary_data)),
            ..Default::default()
        })
    }
}
