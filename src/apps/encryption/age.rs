use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::apps::app_assets::{AppAssets, AppBinary};
use crate::archive::ArchiveExtractor;
use crate::clients::RelgetClient;
use crate::version::AppVersion;

pub struct Age {
    client: Arc<dyn RelgetClient>,
}

impl Age {
    pub const ID: &'static str = "age";
    const OWNER: &'static str = "FiloSottile";
    const REPO: &'static str = "age";
    const EXE_NAME: &'static str = "age";

    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Age {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::new(Self::EXE_NAME)),
            other_bins: vec![AppBinary::new("age-keygen")],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release.find_asset(|a| a.ends_with("-linux-amd64.tar.gz"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data("age", extractor.extract_by_filename("age")?)),
            other_bins: vec![AppBinary::new_with_data(
                "age-keygen",
                extractor.extract_by_filename("age-keygen")?,
            )],
            ..Default::default()
        })
    }
}
