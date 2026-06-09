use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppAssets, AppBinary};
use crate::version::AppVersion;

pub struct Vector {
    client: Arc<GithubClient>,
}

impl Vector {
    pub const ID: &'static str = "vector";
    const OWNER: &'static str = "vectordotdev";
    const REPO: &'static str = "vector";
    const EXE_NAME: &'static str = "vector";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Vector {
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
            a.starts_with("vector-")
                && a.contains("x86_64-unknown-linux-musl")
                && a.ends_with(".tar.gz")
        })?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let binary_data = extractor.extract_by_filename("vector")?;
        Ok(AppAssets {
            binary: Some(AppBinary::new(Self::EXE_NAME, binary_data)),
            ..Default::default()
        })
    }
}
