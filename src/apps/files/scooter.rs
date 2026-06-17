use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::apps::app_assets::{AppAssets, AppBinary};
use crate::archive::ArchiveExtractor;
use crate::clients::RelgetClient;
use crate::version::AppVersion;

pub struct Scooter {
    client: Arc<dyn RelgetClient>,
}

impl Scooter {
    pub const ID: &'static str = "scooter";
    const OWNER: &'static str = "thomasschafer";
    const REPO: &'static str = "scooter";
    const EXE_NAME: &'static str = "scooter";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Scooter {
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
        let name = release
            .find_asset(|a| a.contains("x86_64-unknown-linux-musl") && a.ends_with(".tar.gz"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data(
                Self::EXE_NAME,
                extractor.extract_by_filename(Self::EXE_NAME)?,
            )),
            ..Default::default()
        })
    }
}
