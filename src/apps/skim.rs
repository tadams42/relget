use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::github::GithubClient;
use crate::installer::{run_cmd, with_temp_exe};
use crate::types::{AppBinary, Completion, DownloadedAssets, ManPage};
use crate::version::AppVersion;

pub struct Skim {
    client: Arc<GithubClient>,
}

impl Skim {
    const OWNER: &'static str = "skim-rs";
    const REPO: &'static str = "skim";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Skim {
    fn exe_name(&self) -> &str { "sk" }
    fn url(&self) -> &str { "https://github.com/skim-rs/skim" }

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
            .find(|a| a == "skim-x86_64-unknown-linux-gnu.tar.xz")
            .ok_or_else(|| anyhow!("Can't find skim asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;

        let exe = members
            .iter()
            .find(|m| Path::new(m).file_name().map(|f| f == "sk").unwrap_or(false))
            .cloned()
            .ok_or_else(|| anyhow!("Can't find sk in archive"))?;
        let binary_data = extractor.extract(&exe)?;

        let mut man_pages = Vec::new();
        for man_name in &["sk.1", "sk-tmux.1"] {
            if let Some(m) = members.iter().find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f.to_str().unwrap_or("") == *man_name)
                    .unwrap_or(false)
            }) {
                man_pages.push(ManPage::new(1, *man_name, extractor.extract(m)?));
            }
        }

        let completions = with_temp_exe("sk", &binary_data, |exe_path| {
            Ok(vec![
                Completion::zsh("sk", run_cmd(exe_path, &["--shell", "zsh"])?),
                Completion::bash("sk", run_cmd(exe_path, &["--shell", "bash"])?),
                Completion::fish("sk", run_cmd(exe_path, &["--shell", "fish"])?),
            ])
        })?;

        Ok(DownloadedAssets {
            binary: Some(AppBinary::new("sk", binary_data)),
            man_pages,
            completions,
            ..Default::default()
        })
    }
}
