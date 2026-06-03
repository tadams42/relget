use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::installer::{run_cmd, with_temp_exe};
use crate::types::{AppBinary, Completion, AppAssets, ManPage};
use crate::version::AppVersion;

pub struct Rclone {
    client: Arc<GithubClient>,
}

impl Rclone {
    pub const ID: &'static str = "rclone";
    const OWNER: &'static str = "rclone";
    const REPO: &'static str = "rclone";
    const EXE_NAME: &'static str = "rclone";
    const VERSION_ARG: &'static str = "version";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Rclone {
    fn exe_name(&self) -> &str { Self::EXE_NAME }
    fn cli_version_arg(&self) -> &str { Self::VERSION_ARG }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary:      Some(AppBinary::descriptor(Self::EXE_NAME)),
            man_pages:   vec![ManPage::descriptor(1, "rclone.1")],
            completions: vec![Completion::zsh_desc(Self::EXE_NAME), Completion::bash_desc(Self::EXE_NAME), Completion::fish_desc(Self::EXE_NAME)],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release
            .asset_names()
            .into_iter()
            .find(|a| a.starts_with("rclone-") && a.ends_with("-linux-amd64.zip"))
            .ok_or_else(|| anyhow!("Can't find rclone asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;

        let exe = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "rclone")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find rclone in archive"))?;
        let man = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "rclone.1")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find rclone.1 in archive"))?;

        let binary_data = extractor.extract(&exe)?;
        let man_data = extractor.extract(&man)?;

        let completions = with_temp_exe("rclone", &binary_data, |exe_path| {
            Ok(vec![
                Completion::zsh("rclone", run_cmd(exe_path, &["completion", "zsh", "-"])?),
                Completion::bash("rclone", run_cmd(exe_path, &["completion", "bash", "-"])?),
                Completion::fish("rclone", run_cmd(exe_path, &["completion", "fish", "-"])?),
            ])
        })?;

        Ok(AppAssets {
            binary: Some(AppBinary::new("rclone", binary_data)),
            man_pages: vec![ManPage::new(1, "rclone.1", man_data)],
            completions,
            ..Default::default()
        })
    }
}
