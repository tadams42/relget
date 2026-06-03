use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppBinary, AppAssets};
use crate::version::AppVersion;

pub struct Stylua {
    client: Arc<GithubClient>,
}

impl Stylua {
    pub const ID: &'static str = "stylua";
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

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary:      Some(AppBinary::descriptor(Self::EXE_NAME)),
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release.find_asset(|a| a == "stylua-linux-x86_64.zip")?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        Ok(AppAssets {
            binary: Some(AppBinary::new("stylua", extractor.extract_by_filename("stylua")?)),
            ..Default::default()
        })
    }
}
