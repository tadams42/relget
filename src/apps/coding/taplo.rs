use anyhow::Result;
use std::sync::Arc;

use crate::apps::app_assets::{AppAssets, AppBinary, Completion, Shell};
use crate::apps::{App, gen_completions_subcommand};
use crate::archive::ArchiveExtractor;
use crate::clients::RelgetClient;
use crate::version::AppVersion;

pub struct Taplo {
    client: Arc<dyn RelgetClient>,
}

impl Taplo {
    pub const ID: &'static str = "taplo";
    const OWNER: &'static str = "tamasfe";
    const REPO: &'static str = "taplo";
    const EXE_NAME: &'static str = "taplo";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Taplo {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::new(Self::EXE_NAME)),
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
        let name = release.find_asset(|a| a == "taplo-linux-x86_64.gz")?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;
        let binary_data = extractor.extract(&members[0])?;
        let completions = gen_completions_subcommand("taplo", &binary_data, "completions")?;
        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data(Self::EXE_NAME, binary_data)),
            completions,
            ..Default::default()
        })
    }
}
