use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppBinary, AppAssets};
use crate::version::AppVersion;

pub struct Pgplan {
    client: Arc<GithubClient>,
}

impl Pgplan {
    pub const ID: &'static str = "pgplan";
    const OWNER: &'static str = "JacobArthurs";
    const REPO: &'static str = "pgplan";
    const EXE_NAME: &'static str = "pgplan";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Pgplan {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary:      Some(AppBinary::descriptor(Self::EXE_NAME)),
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release
            .asset_names()
            .into_iter()
            .find(|a| a.as_str() == "pgplan_linux_amd64.tar.gz")
            .ok_or_else(|| anyhow!("Can't find pgplan linux amd64 asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;
        let exe = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "pgplan")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find pgplan in archive"))?;
        let binary_data = extractor.extract(&exe)?;
        Ok(AppAssets {
            binary: Some(AppBinary::new("pgplan", binary_data)),
            ..Default::default()
        })
    }
}
