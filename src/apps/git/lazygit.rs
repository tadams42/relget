use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::github::GithubClient;
use crate::types::{AppBinary, DownloadedAssets};
use crate::version::AppVersion;

pub struct Lazygit {
    client: Arc<GithubClient>,
}

impl Lazygit {
    pub const ID: &'static str = "lazygit";
    pub const DESCRIPTION: &'static str = "Simple terminal UI for git commands";
    pub const URL: &'static str = "https://github.com/jesseduffield/lazygit";
    const OWNER: &'static str = "jesseduffield";
    const REPO: &'static str = "lazygit";
    const EXE_NAME: &'static str = "lazygit";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Lazygit {
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
                a.starts_with("lazygit_")
                    && (a.ends_with("_Linux_x86_64.tar.gz") || a.ends_with("_linux_x86_64.tar.gz"))
            })
            .ok_or_else(|| anyhow!("Can't find lazygit Linux x86_64 asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;
        let exe = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "lazygit")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find lazygit in archive"))?;
        Ok(DownloadedAssets {
            binary: Some(AppBinary::new("lazygit", extractor.extract(&exe)?)),
            ..Default::default()
        })
    }
}
