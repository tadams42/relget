use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppBinary, AppAssets};
use crate::version::AppVersion;

pub struct Sabiql {
    client: Arc<GithubClient>,
}

impl Sabiql {
    pub const ID: &'static str = "sabiql";
    const OWNER: &'static str = "riii111";
    const REPO: &'static str = "sabiql";
    const EXE_NAME: &'static str = "sabiql";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Sabiql {
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
            .find(|a| a.as_str() == "sabiql-x86_64-unknown-linux-gnu.tar.gz")
            .ok_or_else(|| anyhow!("Can't find sabiql linux x86_64 asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;
        let exe = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "sabiql")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find sabiql in archive"))?;
        let binary_data = extractor.extract(&exe)?;
        Ok(AppAssets {
            binary: Some(AppBinary::new("sabiql", binary_data)),
            ..Default::default()
        })
    }
}
