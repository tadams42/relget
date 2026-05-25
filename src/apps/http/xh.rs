use anyhow::{Result, anyhow};
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::github::GithubClient;
use crate::installer::{run_cmd, with_temp_exe};
use crate::types::{AppBinary, Completion, DownloadedAssets, ManPage};
use crate::version::AppVersion;

pub struct Xh {
    client: Arc<GithubClient>,
}

impl Xh {
    pub const ID: &'static str = "xh";
    pub const DESCRIPTION: &'static str = "Friendly and fast HTTP client, HTTPie alternative";
    pub const URL: &'static str = "https://github.com/ducaale/xh";
    const OWNER: &'static str = "ducaale";
    const REPO: &'static str = "xh";
    const EXE_NAME: &'static str = "xh";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Xh {
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
            .find(|a| a.starts_with("xh-") && a.ends_with("-x86_64-unknown-linux-musl.tar.gz"))
            .ok_or_else(|| anyhow!("Can't find xh asset"))?;

        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;

        let exe = members
            .iter()
            .find(|m| {
                std::path::Path::new(m)
                    .file_name()
                    .map(|f| f == "xh")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find xh in archive"))?;

        let binary_data = extractor.extract(&exe)?;

        let (man_pages, completions) = with_temp_exe("xh", &binary_data, |exe_path| {
            let man_data = run_cmd(exe_path, &["--generate", "man"])?;
            let completions = vec![
                Completion::bash("xh", run_cmd(exe_path, &["--generate", "complete-bash"])?),
                Completion::zsh("xh", run_cmd(exe_path, &["--generate", "complete-zsh"])?),
                Completion::fish("xh", run_cmd(exe_path, &["--generate", "complete-fish"])?),
            ];
            Ok((vec![ManPage::new(1, "xh.1", man_data)], completions))
        })?;

        Ok(DownloadedAssets {
            binary: Some(AppBinary::new("xh", binary_data)),
            man_pages,
            completions,
            ..Default::default()
        })
    }
}
