use anyhow::Result;
use std::sync::Arc;

use crate::apps::{App, run_cmd, with_temp_exe};
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppAssets, AppBinary, Completion, ManPage};
use crate::version::AppVersion;

pub struct Fzf {
    client: Arc<GithubClient>,
}

impl Fzf {
    pub const ID: &'static str = "fzf";
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
            binary: Some(AppBinary::descriptor(Self::EXE_NAME)),
            man_pages: vec![
                ManPage::descriptor(1, "fzf.1"),
                ManPage::descriptor(1, "fzf-tmux.1"),
            ],
            completions: vec![
                Completion::zsh_desc(Self::EXE_NAME),
                Completion::bash_desc(Self::EXE_NAME),
                Completion::fish_desc(Self::EXE_NAME),
            ],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;

        let bin_name =
            release.find_asset(|a| a.starts_with("fzf-") && a.ends_with("-linux_amd64.tar.gz"))?;
        let asset = self
            .client
            .download_asset(Self::OWNER, Self::REPO, &bin_name)?;
        let extractor = ArchiveExtractor::new(&bin_name, asset.data);
        let binary_data = extractor.extract_by_filename("fzf")?;

        let completions = with_temp_exe("fzf", &binary_data, |exe_path| {
            Ok(vec![
                Completion::zsh("fzf", run_cmd(exe_path, &["--zsh"])?),
                Completion::bash("fzf", run_cmd(exe_path, &["--bash"])?),
                Completion::fish("fzf", run_cmd(exe_path, &["--fish"])?),
            ])
        })?;

        let tarball = self
            .client
            .download_asset(Self::OWNER, Self::REPO, "tarball")?;
        let tb_name = format!("{}.tar.gz", tarball.name);
        let tb_extractor = ArchiveExtractor::new(&tb_name, tarball.data);
        let mut man_pages = Vec::new();
        for man_name in &["fzf.1", "fzf-tmux.1"] {
            man_pages.push(ManPage::new(
                1,
                *man_name,
                tb_extractor.extract_by_filename(man_name)?,
            ));
        }

        Ok(AppAssets {
            binary: Some(AppBinary::new("fzf", binary_data)),
            man_pages,
            completions,
            ..Default::default()
        })
    }
}
