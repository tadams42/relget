use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::CodebergClient;
use crate::types::{AppAssets, AppBinary};
use crate::version::AppVersion;

pub struct Mergiraf {
    client: Arc<CodebergClient>,
}

impl Mergiraf {
    pub const ID: &'static str = "mergiraf";
    const OWNER: &'static str = "mergiraf";
    const REPO: &'static str = "mergiraf";

    const EXE_NAME: &'static str = "mergiraf";
    pub fn new(client: Arc<CodebergClient>) -> Self { Self { client } }
}

impl App for Mergiraf {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::descriptor(Self::EXE_NAME)),
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release.find_asset(|a| {
            a.contains("x86_64") && a.contains("linux") && a.ends_with(".tar.gz")
        })?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        Ok(AppAssets {
            binary: Some(AppBinary::new("mergiraf", extractor.extract_by_filename("mergiraf")?)),
            ..Default::default()
        })
    }
}
