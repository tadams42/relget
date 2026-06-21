use std::sync::Arc;

use anyhow::Result;

use crate::{App, AppAssets, AppBinary, AppVersion, ArchiveExtractor, ManPage, RelgetClient};

pub struct Jq {
    client: Arc<dyn RelgetClient>,
}

impl Jq {
    pub const ID: &'static str = "jq";
    const OWNER: &'static str = "jqlang";
    const REPO: &'static str = "jq";
    const EXE_NAME: &'static str = "jq";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Jq {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::new(Self::EXE_NAME)),
            man_pages: vec![ManPage::new(1, "jq.1")],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;

        let tar_name = release.find_asset(|a| a.starts_with("jq-") && a.ends_with(".tar.gz"))?;
        let tar_asset = self
            .client
            .download_asset(Self::OWNER, Self::REPO, &tar_name)?;
        let extractor = ArchiveExtractor::new(&tar_name, tar_asset.data);
        let man_data = extractor.extract_by_filename("jq.1")?;

        let bin_name = release.find_asset(|a| a == "jq-linux-amd64")?;
        let bin_asset = self
            .client
            .download_asset(Self::OWNER, Self::REPO, &bin_name)?;

        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data("jq", bin_asset.data)),
            man_pages: vec![ManPage::new_with_data(1, "jq.1", man_data)],
            ..Default::default()
        })
    }
}
