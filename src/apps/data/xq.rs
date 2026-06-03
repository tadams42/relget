use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::installer::gen_completions_subcommand;
use crate::types::{AppBinary, Completion, AppAssets};
use crate::version::AppVersion;

pub struct Xq {
    client: Arc<GithubClient>,
}

impl Xq {
    pub const ID: &'static str = "xq";
    const OWNER: &'static str = "sibprogrammer";
    const REPO: &'static str = "xq";
    const EXE_NAME: &'static str = "xq";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Xq {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary:      Some(AppBinary::descriptor("xq")),
            completions: vec![Completion::zsh_desc("xq"), Completion::bash_desc("xq"), Completion::fish_desc("xq")],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release
            .asset_names()
            .into_iter()
            .find(|a| a.starts_with("xq_") && a.ends_with("_linux_amd64.tar.gz"))
            .ok_or_else(|| anyhow!("Can't find xq asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;
        let exe = members
            .iter()
            .find(|m| Path::new(m).file_name().map(|f| f == "xq").unwrap_or(false))
            .cloned()
            .ok_or_else(|| anyhow!("Can't find xq in archive"))?;
        let binary_data = extractor.extract(&exe)?;
        let completions = gen_completions_subcommand("xq", &binary_data, "completion")?;
        Ok(AppAssets {
            binary: Some(AppBinary::new("xq", binary_data)),
            completions,
            ..Default::default()
        })
    }
}
