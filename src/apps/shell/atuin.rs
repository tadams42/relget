use anyhow::Result;
use std::sync::Arc;

use crate::apps::app_assets::{AppAssets, AppBinary, Completion, Shell};
use crate::apps::{App, gen_completions_shell_flag};
use crate::archive::ArchiveExtractor;
use crate::clients::RelgetClient;
use crate::version::AppVersion;

pub struct Atuin {
    client: Arc<dyn RelgetClient>,
}

impl Atuin {
    pub const ID: &'static str = "atuin";
    const OWNER: &'static str = "atuinsh";
    const REPO: &'static str = "atuin";
    const EXE_NAME: &'static str = "atuin";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Atuin {
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
        let name = release.find_asset(|a| a == "atuin-x86_64-unknown-linux-gnu.tar.gz")?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let binary_data = extractor.extract_by_filename("atuin")?;
        let completions =
            gen_completions_shell_flag("atuin", &binary_data, "gen-completions", "--shell")?;
        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data("atuin", binary_data)),
            completions,
            ..Default::default()
        })
    }
}
