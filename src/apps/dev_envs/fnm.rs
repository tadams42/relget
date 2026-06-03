use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::installer::gen_completions_shell_flag;
use crate::types::{AppBinary, Completion, AppAssets};
use crate::version::AppVersion;

pub struct Fnm {
    client: Arc<GithubClient>,
}

impl Fnm {
    pub const ID: &'static str = "fnm";
    const OWNER: &'static str = "Schniz";
    const REPO: &'static str = "fnm";
    const EXE_NAME: &'static str = "fnm";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Fnm {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary:      Some(AppBinary::descriptor("fnm")),
            completions: vec![Completion::zsh_desc("fnm"), Completion::bash_desc("fnm"), Completion::fish_desc("fnm")],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release
            .asset_names()
            .into_iter()
            .find(|a| a == "fnm-linux.zip")
            .ok_or_else(|| anyhow!("Can't find fnm-linux.zip"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;
        let exe = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "fnm")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find fnm in archive"))?;
        let binary_data = extractor.extract(&exe)?;
        let completions =
            gen_completions_shell_flag("fnm", &binary_data, "completions", "--shell")?;
        Ok(AppAssets {
            binary: Some(AppBinary::new("fnm", binary_data)),
            completions,
            ..Default::default()
        })
    }
}
