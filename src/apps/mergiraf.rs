use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::codeberg::CodebergClient;
use crate::types::{AppBinary, DownloadedAssets};
use crate::version::AppVersion;

pub struct Mergiraf {
    client: Arc<CodebergClient>,
}

impl Mergiraf {
    const OWNER: &'static str = "mergiraf";
    const REPO: &'static str = "mergiraf";

    pub fn new(client: Arc<CodebergClient>) -> Self { Self { client } }
}

impl App for Mergiraf {
    fn exe_name(&self) -> &str { "mergiraf" }
    fn url(&self) -> &str { "https://codeberg.org/mergiraf/mergiraf" }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn download(&self) -> Result<DownloadedAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release
            .asset_names()
            .into_iter()
            .find(|a| a.contains("x86_64") && a.contains("linux") && a.ends_with(".tar.gz"))
            .ok_or_else(|| anyhow!("Can't find mergiraf linux-x86_64 asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;

        let exe_entry = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "mergiraf")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find mergiraf binary in archive"))?;

        Ok(DownloadedAssets {
            binary: Some(AppBinary::new("mergiraf", extractor.extract(&exe_entry)?)),
            ..Default::default()
        })
    }
}
