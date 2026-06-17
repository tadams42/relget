use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::RelgetClient;
use crate::types::{AppAssets, AppBinary, ManPage};
use crate::version::AppVersion;

pub struct Fend {
    client: Arc<dyn RelgetClient>,
}

impl Fend {
    pub const ID: &'static str = "fend";
    const OWNER: &'static str = "printfn";
    const REPO: &'static str = "fend";
    const EXE_NAME: &'static str = "fend";

    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Fend {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::descriptor(Self::EXE_NAME)),
            man_pages: vec![ManPage::descriptor(1, "fend.1")],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;

        let zip_name = release
            .find_asset(|a| a.starts_with("fend-") && a.ends_with("-linux-x86_64-musl.zip"))?;
        let zip_asset = self
            .client
            .download_asset(Self::OWNER, Self::REPO, &zip_name)?;
        let extractor = ArchiveExtractor::new(&zip_name, zip_asset.data);
        let binary_data = extractor.extract_by_filename(Self::EXE_NAME)?;

        let man_asset = self
            .client
            .download_asset(Self::OWNER, Self::REPO, "fend.1")?;

        Ok(AppAssets {
            binary: Some(AppBinary::new(Self::EXE_NAME, binary_data)),
            man_pages: vec![ManPage::new(1, "fend.1", man_asset.data)],
            ..Default::default()
        })
    }
}
