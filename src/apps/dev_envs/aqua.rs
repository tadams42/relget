use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::github::GithubClient;
use crate::installer::gen_completions_subcommand;
use crate::types::{AppBinary, DownloadedAssets};
use crate::version::AppVersion;

pub struct Aqua {
    client: Arc<GithubClient>,
}

impl Aqua {
    pub const ID: &'static str = "aqua";
    pub const DESCRIPTION: &'static str = "Declarative CLI tool installer and version manager";
    pub const URL: &'static str = "https://github.com/aquaproj/aqua";
    const OWNER: &'static str = "aquaproj";
    const REPO: &'static str = "aqua";
    const EXE_NAME: &'static str = "aqua";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Aqua {
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
            .find(|a| a == "aqua_linux_amd64.tar.gz")
            .ok_or_else(|| anyhow!("Can't find aqua_linux_amd64.tar.gz"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;
        let exe = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "aqua")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find aqua in archive"))?;
        let binary_data = extractor.extract(&exe)?;
        let completions = gen_completions_subcommand("aqua", &binary_data, "completion")?;
        Ok(DownloadedAssets {
            binary: Some(AppBinary::new("aqua", binary_data)),
            completions,
            ..Default::default()
        })
    }
}
