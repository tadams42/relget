use std::sync::Arc;

use anyhow::Result;

use crate::{App, AppAssets, AppBinary, AppVersion, ArchiveExtractor, RelgetClient};

pub struct Sttr {
    client: Arc<dyn RelgetClient>,
}

impl Sttr {
    pub const ID: &'static str = "sttr";
    const OWNER: &'static str = "abhimanyu003";
    const REPO: &'static str = "sttr";
    const EXE_NAME: &'static str = "sttr";

    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Sttr {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn cli_version_arg(&self) -> &str { "version" }

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
        let name = release.find_asset(|a| a == "sttr_Linux_x86_64.tar.gz")?;
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
