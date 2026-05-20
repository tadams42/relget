use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::github::GithubClient;
use crate::types::{AppBinary, DownloadedAssets};
use crate::version::AppVersion;

pub struct Difftastic {
    client: Arc<GithubClient>,
}

impl Difftastic {
    const OWNER: &'static str = "Wilfred";
    const REPO: &'static str = "difftastic";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Difftastic {
    fn exe_name(&self) -> &str { "difft" }
    fn url(&self) -> &str { "https://github.com/Wilfred/difftastic" }
    fn installed_version_word_index(&self) -> isize { 1 }

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
            .find(|a| a == "difft-x86_64-unknown-linux-gnu.tar.gz")
            .ok_or_else(|| anyhow!("Can't find difft asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;
        let exe = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "difft")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find difft in archive"))?;
        Ok(DownloadedAssets {
            binary: Some(AppBinary::new("difft", extractor.extract(&exe)?)),
            ..Default::default()
        })
    }
}
