use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::installer::{run_cmd, with_temp_exe};
use crate::types::{AppBinary, Completion, AppAssets, ManPage};
use crate::version::AppVersion;

pub struct Fzf {
    client: Arc<GithubClient>,
}

impl Fzf {
    pub const ID: &'static str = "fzf";
    pub const CATEGORY: &'static str = "shell";
    pub const DESCRIPTION: &'static str = "General-purpose command-line fuzzy finder";
    pub const URL: &'static str = "https://github.com/junegunn/fzf";
    const OWNER: &'static str = "junegunn";
    const REPO: &'static str = "fzf";
    const EXE_NAME: &'static str = "fzf";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Fzf {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary:      Some(AppBinary::descriptor("fzf")),
            man_pages:   vec![
                ManPage::descriptor(1, "fzf.1"),
                ManPage::descriptor(1, "fzf-tmux.1"),
            ],
            completions: vec![Completion::zsh_desc("fzf"), Completion::bash_desc("fzf"), Completion::fish_desc("fzf")],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;

        // Binary
        let bin_name = release
            .asset_names()
            .into_iter()
            .find(|a| a.starts_with("fzf-") && a.ends_with("-linux_amd64.tar.gz"))
            .ok_or_else(|| anyhow!("Can't find fzf binary asset"))?;
        let asset = self
            .client
            .download_asset(Self::OWNER, Self::REPO, &bin_name)?;
        let extractor = ArchiveExtractor::new(&bin_name, asset.data);
        let members = extractor.members()?;
        let exe_entry = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "fzf")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find fzf executable in archive"))?;
        let binary_data = extractor.extract(&exe_entry)?;

        // Completions from binary
        let completions = with_temp_exe("fzf", &binary_data, |exe_path| {
            Ok(vec![
                Completion::zsh("fzf", run_cmd(exe_path, &["--zsh"])?),
                Completion::bash("fzf", run_cmd(exe_path, &["--bash"])?),
                Completion::fish("fzf", run_cmd(exe_path, &["--fish"])?),
            ])
        })?;

        // Man pages from tarball source
        let tarball = self
            .client
            .download_asset(Self::OWNER, Self::REPO, "tarball")?;
        let tb_name = format!("{}.tar.gz", tarball.name);
        let tb_extractor = ArchiveExtractor::new(&tb_name, tarball.data);
        let tb_members = tb_extractor.members()?;
        let mut man_pages = Vec::new();
        for member in &tb_members {
            let fname = Path::new(member)
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("");
            if fname == "fzf.1" || fname == "fzf-tmux.1" {
                let data = tb_extractor.extract(member)?;
                man_pages.push(ManPage::new(1, fname.to_string(), data));
            }
        }

        Ok(AppAssets {
            binary: Some(AppBinary::new("fzf", binary_data)),
            man_pages,
            completions,
            ..Default::default()
        })
    }
}
