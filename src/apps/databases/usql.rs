use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppBinary, AppAssets};
use crate::version::AppVersion;

pub struct Usql {
    client: Arc<GithubClient>,
}

impl Usql {
    pub const ID: &'static str = "usql";
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

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary:      Some(AppBinary::descriptor(Self::EXE_NAME)),
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release.find_asset(|a| a.starts_with("usql_static-") && a.ends_with("-linux-amd64.tar.bz2"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        Ok(AppAssets {
            binary: Some(AppBinary::new("usql", extractor.extract_by_filename("usql_static")?)),
            ..Default::default()
        })
    }
}
