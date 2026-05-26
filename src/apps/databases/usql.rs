use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppBinary, DownloadedAssets};
use crate::version::AppVersion;

pub struct Usql {
    client: Arc<GithubClient>,
}

impl Usql {
    pub const ID: &'static str = "usql";
    pub const DESCRIPTION: &'static str =
        "Universal CLI for PostgreSQL, MySQL, SQLite, and many other databases";
    pub const URL: &'static str = "https://github.com/xo/usql";
    const OWNER: &'static str = "xo";
    const REPO: &'static str = "usql";
    const EXE_NAME: &'static str = "usql";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Usql {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

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
            .find(|a| {
                a.starts_with("usql_static-") && a.ends_with("-linux-amd64.tar.bz2")
            })
            .ok_or_else(|| anyhow!("Can't find usql static linux amd64 asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;
        let exe = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "usql")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find usql in archive"))?;
        let binary_data = extractor.extract(&exe)?;
        Ok(DownloadedAssets {
            binary: Some(AppBinary::new("usql", binary_data)),
            ..Default::default()
        })
    }
}
