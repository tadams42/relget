use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::installer::{run_cmd, with_temp_exe};
use crate::types::{AppBinary, Completion, AppAssets};
use crate::version::AppVersion;

pub struct Delta {
    client: Arc<GithubClient>,
}

impl Delta {
    pub const ID: &'static str = "delta";
    const OWNER: &'static str = "dandavison";
    const REPO: &'static str = "delta";
    const EXE_NAME: &'static str = "delta";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Delta {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary:      Some(AppBinary::descriptor("delta")),
            completions: vec![Completion::zsh_desc("delta"), Completion::bash_desc("delta"), Completion::fish_desc("delta")],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release
            .asset_names()
            .into_iter()
            .find(|a| a.ends_with("x86_64-unknown-linux-musl.tar.gz"))
            .ok_or_else(|| anyhow!("Can't find delta asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;
        let exe = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "delta")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find delta in archive"))?;
        let binary_data = extractor.extract(&exe)?;
        let completions = with_temp_exe("delta", &binary_data, |exe_path| {
            Ok(vec![
                Completion::zsh("delta", run_cmd(exe_path, &["--generate-completion", "zsh"])?),
                Completion::bash("delta", run_cmd(exe_path, &["--generate-completion", "bash"])?),
                Completion::fish("delta", run_cmd(exe_path, &["--generate-completion", "fish"])?),
            ])
        })?;
        Ok(AppAssets {
            binary: Some(AppBinary::new("delta", binary_data)),
            completions,
            ..Default::default()
        })
    }
}
