use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppAssets, AppBinary};
use crate::version::AppVersion;

pub struct D4S {
    client: Arc<GithubClient>,
}

impl D4S {
    pub const ID: &'static str = "d4s";
    const OWNER: &'static str = "jr-k";
    const REPO: &'static str = "d4s";
    const EXE_NAME: &'static str = "d4s";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for D4S {
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
        let name =
            release.find_asset(|a| a.starts_with("d4s") && a.ends_with("linux_amd64.tar.gz"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        Ok(AppAssets {
            binary: Some(AppBinary::new("d4s", extractor.extract_by_filename("d4s")?)),
            ..Default::default()
        })
    }
}
