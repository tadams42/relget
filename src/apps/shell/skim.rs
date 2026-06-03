use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::installer::{run_cmd, with_temp_exe};
use crate::types::{AppBinary, Completion, AppAssets, ManPage};
use crate::version::AppVersion;

pub struct Skim {
    client: Arc<GithubClient>,
}

impl Skim {
    pub const ID: &'static str = "skim";
    const OWNER: &'static str = "skim-rs";
    const REPO: &'static str = "skim";
    const EXE_NAME: &'static str = "sk";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Skim {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary:      Some(AppBinary::descriptor(Self::EXE_NAME)),
            man_pages:   vec![
                ManPage::descriptor(1, "sk.1"),
                ManPage::descriptor(1, "sk-tmux.1"),
            ],
            completions: vec![Completion::zsh_desc(Self::EXE_NAME), Completion::bash_desc(Self::EXE_NAME), Completion::fish_desc(Self::EXE_NAME)],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
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

        Ok(AppAssets {
            binary: Some(AppBinary::new("sk", binary_data)),
            man_pages,
            completions,
            ..Default::default()
        })
    }
}
