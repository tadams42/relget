use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppAssets, AppBinary};
use crate::version::AppVersion;

pub struct Trash {
    client: Arc<GithubClient>,
}

impl Trash {
    pub const ID: &'static str = "trash-cli-rs";
    const OWNER: &'static str = "orf";
    const REPO: &'static str = "trash";
    const EXE_NAME: &'static str = "trash";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Trash {
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
        let name = release.find_asset(|a| a == "trash-Linux-musl-x86_64.tar.gz")?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        Ok(AppAssets {
            binary: Some(AppBinary::new("trash", extractor.extract_by_filename("trash")?)),
            ..Default::default()
        })
    }
}
