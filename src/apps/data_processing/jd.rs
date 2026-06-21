use std::sync::Arc;

use anyhow::Result;

use crate::{App, AppAssets, AppBinary, AppVersion, RelgetClient};

pub struct Jd {
    client: Arc<dyn RelgetClient>,
}

impl Jd {
    pub const ID: &'static str = "jd";
    const OWNER: &'static str = "josephburnett";
    const REPO: &'static str = "jd";
    const EXE_NAME: &'static str = "jd";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Jd {
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
        let name = release.find_asset(|a| a == "jd-amd64-linux")?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data(Self::EXE_NAME, asset.data)),
            ..Default::default()
        })
    }
}
