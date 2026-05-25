use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::github::GithubClient;
use crate::installer::gen_completions_subcommand;
use crate::types::{AppBinary, DownloadedAssets};
use crate::version::AppVersion;

pub struct Gonzo {
    client: Arc<GithubClient>,
}

impl Gonzo {
    pub const ID: &'static str = "gonzo";
    pub const DESCRIPTION: &'static str = "Log viewer TUI for structured and plain-text logs";
    pub const URL: &'static str = "https://github.com/control-theory/gonzo";
    const OWNER: &'static str = "control-theory";
    const REPO: &'static str = "gonzo";
    const EXE_NAME: &'static str = "gonzo";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Gonzo {
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
            .find(|a| a.starts_with("gonzo") && a.ends_with("linux-amd64.tar.gz"))
            .ok_or_else(|| anyhow!("Can't find gonzo asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;
        let exe = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "gonzo")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find gonzo in archive"))?;
        let binary_data = extractor.extract(&exe)?;
        let completions = gen_completions_subcommand("gonzo", &binary_data, "completion")?;
        Ok(DownloadedAssets {
            binary: Some(AppBinary::new("gonzo", binary_data)),
            completions,
            ..Default::default()
        })
    }
}
