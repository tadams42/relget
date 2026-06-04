use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppAssets, AppBinary};
use crate::version::AppVersion;

pub struct Worktrunk {
    client: Arc<GithubClient>,
}

impl Worktrunk {
    pub const ID: &'static str = "worktrunk";
    const OWNER: &'static str = "max-sixty";
    const REPO: &'static str = "worktrunk";
    const EXE_NAME: &'static str = "wt";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Worktrunk {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::descriptor(Self::EXE_NAME)),
            other_bins: vec![AppBinary::descriptor("git-wt")],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release
            .find_asset(|a| a.contains("x86_64-unknown-linux-musl") && a.ends_with(".tar.xz"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        Ok(AppAssets {
            binary: Some(AppBinary::new("wt", extractor.extract_by_filename("wt")?)),
            other_bins: vec![AppBinary::new(
                "git-wt",
                extractor.extract_by_filename("git-wt")?,
            )],
            ..Default::default()
        })
    }
}
