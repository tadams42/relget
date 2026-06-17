use anyhow::Result;
use std::sync::Arc;

use crate::apps::app_assets::{AppAssets, AppBinary, Completion, Shell};
use crate::apps::{App, gen_completions_subcommand};
use crate::clients::RelgetClient;
use crate::version::AppVersion;

pub struct Logdy {
    client: Arc<dyn RelgetClient>,
}

impl Logdy {
    pub const ID: &'static str = "logdy";
    const OWNER: &'static str = "logdyhq";
    const REPO: &'static str = "logdy-core";
    const EXE_NAME: &'static str = "logdy";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Logdy {
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
        let name = release.find_asset(|a| a == "logdy_linux_amd64")?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let completions = gen_completions_subcommand("logdy", &asset.data, "completion")?;
        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data("logdy", asset.data)),
            completions,
            ..Default::default()
        })
    }
}
