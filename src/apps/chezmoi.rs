use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::github::GithubClient;
use crate::installer::gen_completions_subcommand;
use crate::types::{AppBinary, DownloadedAssets};
use crate::version::AppVersion;

pub struct Chezmoi {
    client: Arc<GithubClient>,
}

impl Chezmoi {
    pub const DESCRIPTION: &'static str = "Dotfiles manager across multiple machines";
    pub const URL: &'static str = "https://github.com/twpayne/chezmoi";
    const OWNER: &'static str = "twpayne";
    const REPO: &'static str = "chezmoi";
    const EXE_NAME: &'static str = "chezmoi";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Chezmoi {
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
            // .find(|a| a.starts_with("chezmoi_") && a.ends_with("_linux-glibc_amd64.tar.gz"))
            .find(|a| a.starts_with("chezmoi_") && a.ends_with("_linux-musl_amd64.tar.gz"))
            .ok_or_else(|| anyhow!("Can't find chezmoi asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;
        let exe = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "chezmoi")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find chezmoi in archive"))?;
        let binary_data = extractor.extract(&exe)?;
        let completions = gen_completions_subcommand("chezmoi", &binary_data, "completion")?;
        Ok(DownloadedAssets {
            binary: Some(AppBinary::new("chezmoi", binary_data)),
            completions,
            ..Default::default()
        })
    }
}
