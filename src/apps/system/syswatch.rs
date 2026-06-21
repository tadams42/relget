use std::sync::Arc;

use anyhow::Result;

use crate::{App, AppAssets, AppBinary, AppVersion, ArchiveExtractor, RelgetClient};

pub struct Syswatch {
    client: Arc<dyn RelgetClient>,
}

impl Syswatch {
    pub const ID: &'static str = "syswatch";
    const OWNER: &'static str = "matthart1983";
    const REPO: &'static str = "syswatch";
    const EXE_NAME: &'static str = "syswatch";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Syswatch {
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
        let name = release.find_asset(|a| a == "syswatch-linux-x86_64-static.tar.gz")?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data(
                Self::EXE_NAME,
                extractor.extract_by_filename("syswatch-linux-x86_64-static")?,
            )),
            ..Default::default()
        })
    }
}
