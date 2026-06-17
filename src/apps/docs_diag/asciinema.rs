use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::clients::RelgetClient;
use crate::types::{AppAssets, AppBinary};
use crate::version::AppVersion;

pub struct Asciinema {
    client: Arc<dyn RelgetClient>,
}

impl Asciinema {
    pub const ID: &'static str = "asciinema";
    const OWNER: &'static str = "asciinema";
    const REPO: &'static str = "asciinema";
    const EXE_NAME: &'static str = "asciinema";

    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Asciinema {
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
        let name = release.find_asset(|a| a == "asciinema-x86_64-unknown-linux-musl")?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        Ok(AppAssets {
            binary: Some(AppBinary::new(Self::EXE_NAME, asset.data)),
            ..Default::default()
        })
    }
}
