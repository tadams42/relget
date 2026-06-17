use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::apps::app_assets::{AppAssets, AppBinary};
use crate::clients::RelgetClient;
use crate::version::AppVersion;

pub struct Agg {
    client: Arc<dyn RelgetClient>,
}

impl Agg {
    pub const ID: &'static str = "agg";
    const OWNER: &'static str = "asciinema";
    const REPO: &'static str = "agg";
    const EXE_NAME: &'static str = "agg";

    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Agg {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::new(Self::EXE_NAME)),
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release.find_asset(|a| a == "agg-x86_64-unknown-linux-musl")?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data(Self::EXE_NAME, asset.data)),
            ..Default::default()
        })
    }
}
