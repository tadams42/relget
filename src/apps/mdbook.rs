use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::github::GithubClient;
use crate::installer::gen_completions_subcommand;
use crate::types::{AppBinary, DownloadedAssets};
use crate::version::AppVersion;

pub struct Mdbook {
    client: Arc<GithubClient>,
}

impl Mdbook {
    const OWNER: &'static str = "rust-lang";
    const REPO: &'static str = "mdBook";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Mdbook {
    fn exe_name(&self) -> &str { "mdbook" }
    fn url(&self) -> &str { "https://github.com/rust-lang/mdBook" }

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
            .find(|a| a.starts_with("mdbook-") && a.ends_with("-x86_64-unknown-linux-gnu.tar.gz"))
            .ok_or_else(|| anyhow!("Can't find mdbook asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;
        let exe = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "mdbook")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find mdbook in archive"))?;
        let binary_data = extractor.extract(&exe)?;
        let completions = gen_completions_subcommand("mdbook", &binary_data, "completions")?;
        Ok(DownloadedAssets {
            binary: Some(AppBinary::new("mdbook", binary_data)),
            completions,
            ..Default::default()
        })
    }
}
