use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppBinary, AppAssets};
use crate::version::AppVersion;

pub struct Rsv {
    client: Arc<GithubClient>,
}

impl Rsv {
    pub const ID: &'static str = "rsv";
    const OWNER: &'static str = "ribbondz";
    const REPO: &'static str = "rsv";
    const EXE_NAME: &'static str = "rsv";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Rsv {
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
        let name = release
            .asset_names()
            .into_iter()
            .find(|a| a == "x86_64-unknown-linux-musl.zip")
            .ok_or_else(|| anyhow!("Can't find x86_64-unknown-linux-musl.zip"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;
        let exe = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "rsv")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find rsv in archive"))?;
        Ok(AppAssets {
            binary: Some(AppBinary::new("rsv", extractor.extract(&exe)?)),
            ..Default::default()
        })
    }
}
