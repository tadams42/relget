use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppAssets, AppBinary};
use crate::version::AppVersion;

pub struct Pyrefly {
    client: Arc<GithubClient>,
}

impl Pyrefly {
    pub const ID: &'static str = "pyrefly";
    const OWNER: &'static str = "facebook";
    const REPO: &'static str = "pyrefly";
    const EXE_NAME: &'static str = "pyrefly";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Pyrefly {
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
        let name = release.find_asset(|a| a == "pyrefly-linux-x86_64.tar.gz")?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        Ok(AppAssets {
            binary: Some(AppBinary::new("pyrefly", extractor.extract_by_filename("pyrefly")?)),
            ..Default::default()
        })
    }
}
