use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppAssets, AppBinary};
use crate::version::AppVersion;

pub struct Xan {
    client: Arc<GithubClient>,
}

impl Xan {
    pub const ID: &'static str = "xan";
    const OWNER: &'static str = "medialab";
    const REPO: &'static str = "xan";
    const EXE_NAME: &'static str = "xan";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Xan {
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
            release.find_asset(|a| a == "xan-x86_64-unknown-linux-musl.tar.gz")?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        Ok(AppAssets {
            binary: Some(AppBinary::new(
                Self::EXE_NAME,
                extractor.extract_by_filename(Self::EXE_NAME)?,
            )),
            ..Default::default()
        })
    }
}
