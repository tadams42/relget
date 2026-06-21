use std::sync::Arc;

use anyhow::Result;

use crate::{App, AppAssets, AppBinary, AppVersion, ArchiveExtractor, ManPage, RelgetClient};

pub struct Lnav {
    client: Arc<dyn RelgetClient>,
}

impl Lnav {
    pub const ID: &'static str = "lnav";
    const OWNER: &'static str = "tstack";
    const REPO: &'static str = "lnav";
    const EXE_NAME: &'static str = "lnav";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Lnav {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::new(Self::EXE_NAME)),
            man_pages: vec![ManPage::new(1, "lnav.1")],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release.find_asset(|a| {
            a.starts_with("lnav-") && a.contains("linux-musl-x86_64") && a.ends_with(".zip")
        })?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let binary_data = extractor.extract_by_filename("lnav")?;
        let man_data = extractor.extract_by_filename("lnav.1")?;
        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data(Self::EXE_NAME, binary_data)),
            man_pages: vec![ManPage::new_with_data(1, "lnav.1", man_data)],
            ..Default::default()
        })
    }
}
