use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::github::GithubClient;
use crate::installer::gen_completions_subcommand;
use crate::types::{AppBinary, DownloadedAssets};
use crate::version::AppVersion;

pub struct Starship {
    client: Arc<GithubClient>,
}

impl Starship {
    pub const DESCRIPTION: &'static str =
        "Minimal, blazing-fast, infinitely customizable shell prompt";
    pub const URL: &'static str = "https://github.com/starship/starship";
    const OWNER: &'static str = "starship";
    const REPO: &'static str = "starship";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Starship {
    fn exe_name(&self) -> &str { "starship" }

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
            .find(|a| a == "starship-x86_64-unknown-linux-musl.tar.gz")
            .ok_or_else(|| anyhow!("Can't find starship asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;
        let exe = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "starship")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find starship in archive"))?;
        let binary_data = extractor.extract(&exe)?;
        let completions = gen_completions_subcommand("starship", &binary_data, "completions")?;
        Ok(DownloadedAssets {
            binary: Some(AppBinary::new("starship", binary_data)),
            completions,
            ..Default::default()
        })
    }
}
