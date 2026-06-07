use anyhow::{Result, anyhow};
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppAssets, AppBinary};
use crate::version::AppVersion;

pub struct Rhit {
    client: Arc<GithubClient>,
}

impl Rhit {
    pub const ID: &'static str = "rhit";
    const OWNER: &'static str = "canop";
    const REPO: &'static str = "rhit";
    const EXE_NAME: &'static str = "rhit";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Rhit {
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
        let name = release.find_asset(|a| a.starts_with("rhit_") && a.ends_with(".zip"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let member = extractor
            .members()?
            .into_iter()
            .find(|m| m.contains("x86_64-unknown-linux-musl") && m.ends_with("/rhit"))
            .ok_or_else(|| anyhow!("Can't find rhit musl binary in zip"))?;
        Ok(AppAssets {
            binary: Some(AppBinary::new(Self::EXE_NAME, extractor.extract(&member)?)),
            ..Default::default()
        })
    }
}
