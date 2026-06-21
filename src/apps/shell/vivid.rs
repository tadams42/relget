use std::sync::Arc;

use anyhow::Result;

use crate::{App, AppAssets, AppBinary, AppVersion, ArchiveExtractor, RelgetClient};

pub struct Vivid {
    client: Arc<dyn RelgetClient>,
}

impl Vivid {
    pub const ID: &'static str = "vivid";
    const OWNER: &'static str = "sharkdp";
    const REPO: &'static str = "vivid";
    const EXE_NAME: &'static str = "vivid";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Vivid {
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
        let name = release.find_asset(|a| {
            a.starts_with("vivid-") && a.ends_with("-x86_64-unknown-linux-musl.tar.gz")
        })?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let binary_data = extractor.extract_by_filename(Self::EXE_NAME)?;
        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data(Self::EXE_NAME, binary_data)),
            ..Default::default()
        })
    }
}
