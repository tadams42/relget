use anyhow::Result;
use std::sync::Arc;

use crate::apps::{App, gen_completions_subcommand};
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppAssets, AppBinary, Completion, ManPage};
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
            binary: Some(AppBinary::descriptor(Self::EXE_NAME)),
            man_pages: vec![
                ManPage::descriptor(1, "sk.1"),
                ManPage::descriptor(1, "sk-tmux.1"),
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
        let name = release.find_asset(|a| a == "skim-x86_64-unknown-linux-gnu.tar.xz")?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let binary_data = extractor.extract_by_filename("sk")?;

        let mut man_pages = Vec::new();
        for man_name in &["sk.1", "sk-tmux.1"] {
            if let Ok(data) = extractor.extract_by_filename(man_name) {
                man_pages.push(ManPage::new(1, *man_name, data));
            }
        }

        let completions = gen_completions_subcommand("sk", &binary_data, "--shell")?;

        Ok(AppAssets {
            binary: Some(AppBinary::new("sk", binary_data)),
            man_pages,
            completions,
            ..Default::default()
        })
    }
}
