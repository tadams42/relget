use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::github::GithubClient;
use crate::installer::{run_cmd, with_temp_exe};
use crate::types::{AppBinary, Completion, DownloadedAssets, ManPage};
use crate::version::AppVersion;

pub struct FdFind {
    client: Arc<GithubClient>,
}

impl FdFind {
    pub const DESCRIPTION: &'static str = "Simple, fast, user-friendly alternative to find";
    pub const URL: &'static str = "https://github.com/sharkdp/fd";
    const OWNER: &'static str = "sharkdp";
    const REPO: &'static str = "fd";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for FdFind {
    fn exe_name(&self) -> &str { "fd" }

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
            .find(|a| a.starts_with("fd-") && a.ends_with("-x86_64-unknown-linux-musl.tar.gz"))
            .ok_or_else(|| anyhow!("Can't find fd asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;

        let exe = members
            .iter()
            .find(|m| Path::new(m).file_name().map(|f| f == "fd").unwrap_or(false))
            .cloned()
            .ok_or_else(|| anyhow!("Can't find fd in archive"))?;
        let man = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "fd.1")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find fd.1 in archive"))?;

        let binary_data = extractor.extract(&exe)?;
        let man_data = extractor.extract(&man)?;

        let completions = with_temp_exe("fd", &binary_data, |exe_path| {
            Ok(vec![
                Completion::zsh("fd", run_cmd(exe_path, &["--gen-completions", "zsh"])?),
                Completion::bash("fd", run_cmd(exe_path, &["--gen-completions", "bash"])?),
                Completion::fish("fd", run_cmd(exe_path, &["--gen-completions", "fish"])?),
            ])
        })?;

        Ok(DownloadedAssets {
            binary: Some(AppBinary::new("fd", binary_data)),
            man_pages: vec![ManPage::new(1, "fd.1", man_data)],
            completions,
            ..Default::default()
        })
    }
}
