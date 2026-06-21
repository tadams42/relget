use std::sync::Arc;

use anyhow::Result;

use crate::{App, AppAssets, AppBinary, AppVersion, ArchiveExtractor, RelgetClient};

pub struct Scc {
    client: Arc<dyn RelgetClient>,
}

impl Scc {
    pub const ID: &'static str = "scc";
    const OWNER: &'static str = "boyter";
    const REPO: &'static str = "scc";
    const EXE_NAME: &'static str = "scc";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Scc {
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
        let name = release.find_asset(|a| a == "scc_Linux_x86_64.tar.gz")?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data("scc", extractor.extract_by_filename("scc")?)),
            ..Default::default()
        })
    }
}
