use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::apps::app_assets::{AppAssets, AppBinary};
use crate::clients::RelgetClient;
use crate::version::AppVersion;

pub struct Mkcert {
    client: Arc<dyn RelgetClient>,
}

impl Mkcert {
    pub const ID: &'static str = "mkcert";
    const OWNER: &'static str = "FiloSottile";
    const REPO: &'static str = "mkcert";
    const EXE_NAME: &'static str = "mkcert";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Mkcert {
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
        let name =
            release.find_asset(|a| a.starts_with("mkcert-") && a.ends_with("-linux-amd64"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data(Self::EXE_NAME, asset.data)),
            ..Default::default()
        })
    }
}
