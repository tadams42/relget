use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::clients::RelgetClient;
use crate::types::{AppAssets, AppBinary, ManPage};
use crate::version::AppVersion;

pub struct Jaq {
    client: Arc<dyn RelgetClient>,
}

impl Jaq {
    pub const ID: &'static str = "jaq";
    const OWNER: &'static str = "01mf02";
    const REPO: &'static str = "jaq";
    const EXE_NAME: &'static str = "jaq";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Jaq {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::descriptor(Self::EXE_NAME)),
            man_pages: vec![ManPage::descriptor(1, "jaq.1")],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;

        let bin_name = release.find_asset(|a| a == "jaq-x86_64-unknown-linux-musl")?;
        let bin_asset = self
            .client
            .download_asset(Self::OWNER, Self::REPO, &bin_name)?;

        let man_name = release.find_asset(|a| a == "jaq.1")?;
        let man_asset = self
            .client
            .download_asset(Self::OWNER, Self::REPO, &man_name)?;

        Ok(AppAssets {
            binary: Some(AppBinary::new(Self::EXE_NAME, bin_asset.data)),
            man_pages: vec![ManPage::new(1, "jaq.1", man_asset.data)],
            ..Default::default()
        })
    }
}
