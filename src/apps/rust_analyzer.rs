use anyhow::{Result, anyhow};
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::github::GithubClient;
use crate::types::{AppBinary, DownloadedAssets};
use crate::version::AppVersion;

pub struct RustAnalyzer {
    client: Arc<GithubClient>,
}

impl RustAnalyzer {
    const OWNER: &'static str = "rust-lang";
    const REPO: &'static str = "rust-analyzer";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for RustAnalyzer {
    fn exe_name(&self) -> &str { "rust-analyzer" }
    fn url(&self) -> &str { "https://github.com/rust-lang/rust-analyzer" }
    fn installed_version_word_index(&self) -> isize { -2 }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn parse_installed_version(&self, data: &str) -> Option<AppVersion> {
        // "rust-analyzer 2024-01-15 ..."
        let normalized = data.replace('-', " ");
        let words: Vec<&str> = normalized.split_whitespace().collect();
        let idx = (words.len() as isize + self.installed_version_word_index()) as usize;
        words.get(idx).and_then(|w| AppVersion::parse(w))
    }

    fn download(&self) -> Result<DownloadedAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release
            .asset_names()
            .into_iter()
            .find(|a| a == "rust-analyzer-x86_64-unknown-linux-gnu.gz")
            .ok_or_else(|| anyhow!("Can't find rust-analyzer asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        // Single .gz file decompresses to one file; member name = name without .gz
        let member = "rust-analyzer-x86_64-unknown-linux-gnu";
        let binary_data = extractor.extract(member)?;
        Ok(DownloadedAssets {
            binary: Some(AppBinary::new("rust-analyzer", binary_data)),
            ..Default::default()
        })
    }
}
