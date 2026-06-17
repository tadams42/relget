use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::apps::app_assets::{AppAssets, AppBinary, ManPage};
use crate::archive::ArchiveExtractor;
use crate::clients::RelgetClient;
use crate::version::AppVersion;

pub struct D2 {
    client: Arc<dyn RelgetClient>,
}

impl D2 {
    pub const ID: &'static str = "d2";
    const OWNER: &'static str = "terrastruct";
    const REPO: &'static str = "d2";
    const EXE_NAME: &'static str = "d2";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for D2 {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::new(Self::EXE_NAME)),
            man_pages: vec![ManPage::new(1, "d2.1")],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name =
            release.find_asset(|a| a.starts_with("d2-") && a.ends_with("-linux-amd64.tar.gz"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let e = ArchiveExtractor::new(&name, asset.data);
        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data(Self::EXE_NAME, e.extract_by_filename("d2")?)),
            man_pages: vec![ManPage::new_with_data(
                1,
                "d2.1",
                e.extract_by_filename("d2.1")?,
            )],
            ..Default::default()
        })
    }
}
