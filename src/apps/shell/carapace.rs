use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppBinary, AppAssets};
use crate::version::AppVersion;

pub struct Carapace {
    client: Arc<GithubClient>,
}

impl Carapace {
    pub const ID: &'static str = "carapace";
    pub const CATEGORY: &'static str = "shell";
    pub const DESCRIPTION: &'static str = "Multi-shell completion generator for command-line tools";
    pub const URL: &'static str = "https://github.com/carapace-sh/carapace-bin";
    const OWNER: &'static str = "carapace-sh";
    const REPO: &'static str = "carapace-bin";
    const EXE_NAME: &'static str = "carapace";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Carapace {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary:      Some(AppBinary::descriptor("carapace")),
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release
            .asset_names()
            .into_iter()
            .find(|a| a.starts_with("carapace") && a.ends_with("linux_amd64.tar.gz"))
            .ok_or_else(|| anyhow!("Can't find carapace asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;
        let exe = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "carapace")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find carapace in archive"))?;
        Ok(AppAssets {
            binary: Some(AppBinary::new("carapace", extractor.extract(&exe)?)),
            ..Default::default()
        })
    }
}
