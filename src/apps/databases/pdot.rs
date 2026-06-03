use anyhow::{Result, anyhow};
use std::sync::Arc;

use crate::apps::App;
use crate::clients::GitlabClient;
use crate::types::{AppBinary, AppAssets};
use crate::version::AppVersion;

pub struct Pdot {
    client: Arc<GitlabClient>,
}

impl Pdot {
    pub const ID: &'static str = "pdot";
    const OWNER: &'static str = "dmfay";
    const REPO: &'static str = "pdot";
    const EXE_NAME: &'static str = "pdot";

    pub fn new(client: Arc<GitlabClient>) -> Self { Self { client } }
}

impl App for Pdot {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary:      Some(AppBinary::descriptor("pdot")),
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release
            .asset_names()
            .into_iter()
            .find(|a| a.contains("Linux x86_64"))
            .ok_or_else(|| anyhow!("Can't find pdot Linux x86_64 asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        Ok(AppAssets {
            binary: Some(AppBinary::new("pdot", asset.data)),
            ..Default::default()
        })
    }
}
