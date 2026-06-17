use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::apps::app_assets::{AppAssets, AppBinary, Completion, ManPage, Shell};
use crate::archive::ArchiveExtractor;
use crate::clients::RelgetClient;
use crate::version::AppVersion;

pub struct Tabiew {
    client: Arc<dyn RelgetClient>,
}

impl Tabiew {
    pub const ID: &'static str = "tabiew";
    const OWNER: &'static str = "shshemi";
    const REPO: &'static str = "tabiew";
    const EXE_NAME: &'static str = "tw";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Tabiew {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::new(Self::EXE_NAME)),
            man_pages: vec![ManPage::new(1, "tabiew.1")],
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

        let bin_name = release.find_asset(|a| a == "tw-x86_64-unknown-linux-musl")?;
        let bin_asset = self
            .client
            .download_asset(Self::OWNER, Self::REPO, &bin_name)?;

        let extras_name = release.find_asset(|a| a == "tabiew-manual-and-completions.tar.gz")?;
        let extras_asset = self
            .client
            .download_asset(Self::OWNER, Self::REPO, &extras_name)?;
        let extras = ArchiveExtractor::new(&extras_name, extras_asset.data);

        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data(Self::EXE_NAME, bin_asset.data)),
            man_pages: vec![ManPage::new_with_data(
                1,
                "tabiew.1",
                extras.extract("manual/tabiew.1")?,
            )],
            completions: vec![
                Completion::new_with_data(
                    Shell::Zsh,
                    Self::EXE_NAME,
                    extras.extract("completion/_tw")?,
                ),
                Completion::new_with_data(
                    Shell::Bash,
                    Self::EXE_NAME,
                    extras.extract("completion/tw.bash")?,
                ),
                Completion::new_with_data(
                    Shell::Fish,
                    Self::EXE_NAME,
                    extras.extract("completion/tw.fish")?,
                ),
            ],
            ..Default::default()
        })
    }
}
