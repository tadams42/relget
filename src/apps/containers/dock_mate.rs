use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::clients::GithubClient;
use crate::types::{AppAssets, AppBinary};
use crate::version::AppVersion;

pub struct DockMate {
    client: Arc<GithubClient>,
}

impl DockMate {
    pub const ID: &'static str = "dockmate";
    const OWNER: &'static str = "shubh-io";
    const REPO: &'static str = "DockMate";
    const EXE_NAME: &'static str = "dockmate";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for DockMate {
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
        let name = release.find_asset(|a| a == "dockmate-linux-amd64")?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        Ok(AppAssets {
            binary: Some(AppBinary::new("dockmate", asset.data)),
            ..Default::default()
        })
    }
}
