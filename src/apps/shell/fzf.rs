use std::sync::Arc;

use anyhow::Result;

use crate::apps::{run_cmd, with_temp_exe};
use crate::{
    App, AppAssets, AppBinary, AppVersion, ArchiveExtractor, Completion, ManPage, RelgetClient,
    Shell,
};

pub struct Fzf {
    client: Arc<dyn RelgetClient>,
}

impl Fzf {
    pub const ID: &'static str = "fzf";
    const OWNER: &'static str = "junegunn";
    const REPO: &'static str = "fzf";
    const EXE_NAME: &'static str = "fzf";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
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
            binary: Some(AppBinary::new(Self::EXE_NAME)),
            man_pages: vec![ManPage::new(1, "fzf.1"), ManPage::new(1, "fzf-tmux.1")],
            completions: vec![
                Completion::new(Shell::Zsh, Self::EXE_NAME),
                Completion::new(Shell::Bash, Self::EXE_NAME),
                Completion::new(Shell::Fish, Self::EXE_NAME),
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
                Completion::new_with_data(Shell::Zsh, "fzf", run_cmd(exe_path, &["--zsh"])?),
                Completion::new_with_data(Shell::Bash, "fzf", run_cmd(exe_path, &["--bash"])?),
                Completion::new_with_data(Shell::Fish, "fzf", run_cmd(exe_path, &["--fish"])?),
            ])
        })?;

        let tarball = self
            .client
            .download_asset(Self::OWNER, Self::REPO, "tarball")?;
        let tb_name = format!("{}.tar.gz", tarball.name);
        let tb_extractor = ArchiveExtractor::new(&tb_name, tarball.data);
        let mut man_pages = Vec::new();
        for man_name in &["fzf.1", "fzf-tmux.1"] {
            man_pages.push(ManPage::new_with_data(
                1,
                *man_name,
                tb_extractor.extract_by_filename(man_name)?,
            ));
        }

        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data("fzf", binary_data)),
            man_pages,
            completions,
            ..Default::default()
        })
    }
}
