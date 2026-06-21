use std::sync::Arc;

use anyhow::{Result, anyhow};

use crate::{App, AppAssets, AppBinary, AppVersion, ArchiveExtractor, RelgetClient};

pub struct Tv {
    client: Arc<dyn RelgetClient>,
}

impl Tv {
    pub const ID: &'static str = "tv";
    const OWNER: &'static str = "alexhallam";
    const REPO: &'static str = "tv";
    const EXE_NAME: &'static str = "tidy-viewer";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Tv {
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
        let name =
            release.find_asset(|a| a.starts_with("tidy-viewer_") && a.ends_with("_amd64.deb"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let deb = ArchiveExtractor::new(&name, asset.data);
        let deb_members = deb.members()?;
        let data_tar = deb_members
            .iter()
            .find(|m| m.starts_with("data.tar"))
            .cloned()
            .ok_or_else(|| anyhow!("Can't find data.tar in deb"))?;
        let data = ArchiveExtractor::new(&data_tar, deb.extract(&data_tar)?);
        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data(
                Self::EXE_NAME,
                data.extract_by_filename(Self::EXE_NAME)?,
            )),
            ..Default::default()
        })
    }
}
