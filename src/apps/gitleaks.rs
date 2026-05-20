use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::github::GithubClient;
use crate::installer::gen_completions_subcommand;
use crate::types::{AppBinary, DownloadedAssets};
use crate::version::AppVersion;

pub struct Gitleaks {
    client: Arc<GithubClient>,
}

impl Gitleaks {
    const OWNER: &'static str = "gitleaks";
    const REPO: &'static str = "gitleaks";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Gitleaks {
    fn exe_name(&self) -> &str { "gitleaks" }
    fn url(&self) -> &str { "https://github.com/gitleaks/gitleaks" }

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
            .find(|a| a.starts_with("gitleaks_") && a.ends_with("_linux_x64.tar.gz"))
            .ok_or_else(|| anyhow!("Can't find gitleaks asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;
        let exe = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "gitleaks")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find gitleaks in archive"))?;
        let binary_data = extractor.extract(&exe)?;
        let completions = gen_completions_subcommand("gitleaks", &binary_data, "completion")?;
        Ok(DownloadedAssets {
            binary: Some(AppBinary::new("gitleaks", binary_data)),
            completions,
            ..Default::default()
        })
    }
}
