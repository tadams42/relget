use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppBinary, AppAssets};
use crate::version::AppVersion;

pub struct Age {
    client: Arc<GithubClient>,
}

impl Age {
    pub const ID: &'static str = "age";
    const OWNER: &'static str = "FiloSottile";
    const REPO: &'static str = "age";
    const EXE_NAME: &'static str = "age";

    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
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
            binary:     Some(AppBinary::descriptor(Self::EXE_NAME)),
            other_bins: vec![AppBinary::descriptor("age-keygen")],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release
            .asset_names()
            .into_iter()
            .find(|a| a.ends_with("-linux-amd64.tar.gz"))
            .ok_or_else(|| anyhow!("Can't find age asset"))?;

        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;

        let age_path = members
            .iter()
            .find(|m| Path::new(m).file_name().map(|f| f == "age").unwrap_or(false))
            .cloned()
            .ok_or_else(|| anyhow!("Can't find age in archive"))?;

        let age_keygen_path = members
            .iter()
            .find(|m| Path::new(m).file_name().map(|f| f == "age-keygen").unwrap_or(false))
            .cloned()
            .ok_or_else(|| anyhow!("Can't find age-keygen in archive"))?;

        let age_data = extractor.extract(&age_path)?;
        let age_keygen_data = extractor.extract(&age_keygen_path)?;

        Ok(AppAssets {
            binary: Some(AppBinary::new("age", age_data)),
            other_bins: vec![AppBinary::new("age-keygen", age_keygen_data)],
            ..Default::default()
        })
    }
}
