use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::github::GithubClient;
use crate::installer::gen_completions_subcommand;
use crate::types::{AppBinary, DownloadedAssets, ManPage};
use crate::version::AppVersion;

pub struct Yq {
    client: Arc<GithubClient>,
}

impl Yq {
    pub const ID: &'static str = "yq";
    pub const DESCRIPTION: &'static str =
        "Portable command-line YAML, JSON, XML, and CSV processor";
    pub const URL: &'static str = "https://github.com/mikefarah/yq";
    const OWNER: &'static str = "mikefarah";
    const REPO: &'static str = "yq";
    const EXE_NAME: &'static str = "yq";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Yq {
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
            .find(|a| a == "yq_linux_amd64.tar.gz")
            .ok_or_else(|| anyhow!("Can't find yq_linux_amd64.tar.gz"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;

        let exe = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "yq_linux_amd64")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find yq_linux_amd64 in archive"))?;
        let man = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "yq.1")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find yq.1 in archive"))?;

        let binary_data = extractor.extract(&exe)?;
        let man_data = extractor.extract(&man)?;
        let completions = gen_completions_subcommand("yq", &binary_data, "completion")?;

        Ok(DownloadedAssets {
            binary: Some(AppBinary::new("yq", binary_data)),
            man_pages: vec![ManPage::new(1, "yq.1", man_data)],
            completions,
            ..Default::default()
        })
    }
}
