use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::clients::GithubClient;
use crate::types::{AppBinary, AppAssets};
use crate::version::AppVersion;

pub struct Fx {
    client: Arc<GithubClient>,
}

impl Fx {
    pub const ID: &'static str = "fx";
    const OWNER: &'static str = "antonmedv";
    const REPO: &'static str = "fx";
    const EXE_NAME: &'static str = "fx";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Fx {
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
        let name = release.find_asset(|a| a == "fx_linux_amd64")?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        Ok(AppAssets {
            binary: Some(AppBinary::new("fx", asset.data)),
            ..Default::default()
        })
    }
}
