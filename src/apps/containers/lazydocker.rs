use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::github::GithubClient;
use crate::types::{AppBinary, DownloadedAssets};
use crate::version::AppVersion;

pub struct LazyDocker {
    client: Arc<GithubClient>,
}

impl LazyDocker {
    pub const ID: &'static str = "lazydocker";
    pub const DESCRIPTION: &'static str = "Terminal UI for Docker containers, images, and compose";
    pub const URL: &'static str = "https://github.com/jesseduffield/lazydocker";
    const OWNER: &'static str = "jesseduffield";
    const REPO: &'static str = "lazydocker";
    const EXE_NAME: &'static str = "lazydocker";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for LazyDocker {
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
            .find(|a| a.starts_with("lazydocker") && a.ends_with("inux_x86_64.tar.gz"))
            .ok_or_else(|| anyhow!("Can't find lazydocker Linux x86_64 asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;
        let exe = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "lazydocker")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find lazydocker in archive"))?;
        Ok(DownloadedAssets {
            binary: Some(AppBinary::new("lazydocker", extractor.extract(&exe)?)),
            ..Default::default()
        })
    }
}
