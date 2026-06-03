use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::installer::{run_cmd, with_temp_exe};
use crate::types::{AppBinary, Completion, AppAssets, ManPage};
use crate::version::AppVersion;

pub struct Ripgrep {
    client: Arc<GithubClient>,
}

impl Ripgrep {
    pub const ID: &'static str = "ripgrep";
    pub const CATEGORY: &'static str = "files";
    pub const DESCRIPTION: &'static str = "Recursive regex search, a faster grep (ripgrep)";
    pub const URL: &'static str = "https://github.com/BurntSushi/ripgrep";
    const OWNER: &'static str = "BurntSushi";
    const REPO: &'static str = "ripgrep";
    const EXE_NAME: &'static str = "rg";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Ripgrep {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary:      Some(AppBinary::descriptor("rg")),
            man_pages:   vec![ManPage::descriptor(1, "rg.1")],
            completions: vec![Completion::zsh_desc("rg"), Completion::bash_desc("rg"), Completion::fish_desc("rg")],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release
            .asset_names()
            .into_iter()
            .find(|a| a.starts_with("ripgrep-") && a.ends_with("-x86_64-unknown-linux-musl.tar.gz"))
            .ok_or_else(|| anyhow!("Can't find ripgrep musl asset"))?;

        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;

        let exe = members
            .iter()
            .find(|m| Path::new(m).file_name().map(|f| f == "rg").unwrap_or(false))
            .cloned()
            .ok_or_else(|| anyhow!("Can't find rg in archive"))?;
        let man = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "rg.1")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find rg.1 in archive"))?;

        let binary_data = extractor.extract(&exe)?;
        let man_data = extractor.extract(&man)?;

        let completions = with_temp_exe("rg", &binary_data, |exe| {
            Ok(vec![
                Completion::zsh("rg", run_cmd(exe, &["--generate", "complete-zsh"])?),
                Completion::bash("rg", run_cmd(exe, &["--generate", "complete-bash"])?),
                Completion::fish("rg", run_cmd(exe, &["--generate", "complete-fish"])?),
            ])
        })?;

        Ok(AppAssets {
            binary: Some(AppBinary::new("rg", binary_data)),
            man_pages: vec![ManPage::new(1, "rg.1", man_data)],
            completions,
            ..Default::default()
        })
    }
}
