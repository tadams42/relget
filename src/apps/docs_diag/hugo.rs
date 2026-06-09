use anyhow::{Context, Result};
use std::sync::Arc;

use crate::apps::{App, gen_completions_subcommand, run_cmd, with_temp_exe};
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppAssets, AppBinary, Completion, ManPage};
use crate::version::AppVersion;

pub struct Hugo {
    client: Arc<GithubClient>,
}

impl Hugo {
    pub const ID: &'static str = "hugo";
    const OWNER: &'static str = "gohugoio";
    const REPO: &'static str = "hugo";
    const EXE_NAME: &'static str = "hugo";

    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Hugo {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn cli_version_arg(&self) -> &str { "version" }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary:      Some(AppBinary::descriptor(Self::EXE_NAME)),
            man_pages:   vec![ManPage::descriptor(1, "hugo.1")],
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
        let name = release.find_asset(|a| {
            a.starts_with("hugo_")
                && !a.contains("extended")
                && a.ends_with("_linux-amd64.tar.gz")
        })?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let binary_data = extractor.extract_by_filename("hugo")?;

        let (completions, man_pages) = with_temp_exe("hugo", &binary_data, |exe_path| {
            let completions =
                gen_completions_subcommand("hugo", &binary_data, "completion")?;
            let man_tmp = tempfile::tempdir()?;
            let man_dir = man_tmp.path().to_str().context("non-UTF8 path")?;
            run_cmd(exe_path, &["gen", "man", "--dir", man_dir])?;
            let man_data = std::fs::read(man_tmp.path().join("hugo.1"))?;
            Ok((completions, vec![ManPage::new(1, "hugo.1", man_data)]))
        })?;

        Ok(AppAssets {
            binary: Some(AppBinary::new("hugo", binary_data)),
            completions,
            man_pages,
            ..Default::default()
        })
    }
}
