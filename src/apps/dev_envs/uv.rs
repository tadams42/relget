use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::installer::{run_cmd, with_temp_exe};
use crate::types::{AppBinary, Completion, AppAssets};
use crate::version::AppVersion;

pub struct Uv {
    client: Arc<GithubClient>,
}

impl Uv {
    pub const ID: &'static str = "uv";
    const OWNER: &'static str = "astral-sh";
    const REPO: &'static str = "uv";
    const EXE_NAME: &'static str = "uv";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Uv {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary:      Some(AppBinary::descriptor(Self::EXE_NAME)),
            other_bins:  vec![AppBinary::descriptor("uvx")],
            completions: vec![
                Completion::zsh_desc(Self::EXE_NAME),
                Completion::bash_desc(Self::EXE_NAME),
                Completion::fish_desc(Self::EXE_NAME),
                Completion::zsh_desc("uvx"),
                Completion::bash_desc("uvx"),
                Completion::fish_desc("uvx"),
            ],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release
            .asset_names()
            .into_iter()
            .find(|a| a == "uv-x86_64-unknown-linux-musl.tar.gz")
            .ok_or_else(|| anyhow!("Can't find uv asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;

        let uv_entry = members
            .iter()
            .find(|m| Path::new(m).file_name().map(|f| f == "uv").unwrap_or(false))
            .cloned()
            .ok_or_else(|| anyhow!("Can't find uv in archive"))?;
        let uvx_entry = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "uvx")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find uvx in archive"))?;

        let uv_data = extractor.extract(&uv_entry)?;
        let uvx_data = extractor.extract(&uvx_entry)?;

        let completions = with_temp_exe("uv", &uv_data, |exe_path| {
            // Also write uvx next to uv for uvx completions
            let uvx_path = exe_path.parent().unwrap().join("uvx");
            std::fs::write(&uvx_path, &uvx_data)?;
            std::fs::set_permissions(&uvx_path, std::fs::Permissions::from_mode(0o755))?;

            let comps = vec![
                Completion::zsh("uv", run_cmd(exe_path, &["generate-shell-completion", "zsh"])?),
                Completion::bash("uv", run_cmd(exe_path, &["generate-shell-completion", "bash"])?),
                Completion::fish("uv", run_cmd(exe_path, &["generate-shell-completion", "fish"])?),
                // uvx completions via uv --generate-shell-completion
                Completion::zsh("uvx", run_cmd(exe_path, &["--generate-shell-completion", "zsh"])?),
                Completion::bash(
                    "uvx",
                    run_cmd(exe_path, &["--generate-shell-completion", "bash"])?,
                ),
                Completion::fish(
                    "uvx",
                    run_cmd(exe_path, &["--generate-shell-completion", "fish"])?,
                ),
            ];
            Ok(comps)
        })?;

        Ok(AppAssets {
            binary: Some(AppBinary::new("uv", uv_data)),
            other_bins: vec![AppBinary::new("uvx", uvx_data)],
            completions,
            ..Default::default()
        })
    }
}

use std::os::unix::fs::PermissionsExt;
