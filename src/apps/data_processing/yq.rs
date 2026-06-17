use anyhow::Result;
use std::sync::Arc;

use crate::apps::app_assets::{AppAssets, AppBinary, Completion, ManPage, Shell};
use crate::apps::{App, gen_completions_subcommand};
use crate::archive::ArchiveExtractor;
use crate::clients::RelgetClient;
use crate::version::AppVersion;

pub struct Yq {
    client: Arc<dyn RelgetClient>,
}

impl Yq {
    pub const ID: &'static str = "yq";
    const OWNER: &'static str = "mikefarah";
    const REPO: &'static str = "yq";
    const EXE_NAME: &'static str = "yq";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Yq {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::new(Self::EXE_NAME)),
            man_pages: vec![ManPage::new(1, "yq.1")],
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
        let name = release.find_asset(|a| a == "yq_linux_amd64.tar.gz")?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let binary_data = extractor.extract_by_filename("yq_linux_amd64")?;
        let man_data = extractor.extract_by_filename("yq.1")?;
        let completions = gen_completions_subcommand("yq", &binary_data, "completion")?;
        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data("yq", binary_data)),
            man_pages: vec![ManPage::new_with_data(1, "yq.1", man_data)],
            completions,
            ..Default::default()
        })
    }
}
