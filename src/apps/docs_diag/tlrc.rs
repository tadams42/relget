use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::apps::app_assets::{AppAssets, AppBinary, Completion, ManPage, Shell};
use crate::archive::ArchiveExtractor;
use crate::clients::RelgetClient;
use crate::version::AppVersion;

pub struct Tlrc {
    client: Arc<dyn RelgetClient>,
}

impl Tlrc {
    pub const ID: &'static str = "tlrc";
    const OWNER: &'static str = "tldr-pages";
    const REPO: &'static str = "tlrc";
    const EXE_NAME: &'static str = "tldr";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Tlrc {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::new(Self::EXE_NAME)),
            man_pages: vec![ManPage::new(1, "tldr.1")],
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
        let name = release.find_asset(|a| {
            a.starts_with("tlrc-") && a.ends_with("-x86_64-unknown-linux-gnu.tar.gz")
        })?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let e = ArchiveExtractor::new(&name, asset.data);
        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data("tldr", e.extract_by_filename("tldr")?)),
            man_pages: vec![ManPage::new_with_data(
                1,
                "tldr.1",
                e.extract_by_filename("tldr.1")?,
            )],
            completions: vec![
                Completion::new_with_data(Shell::Bash, "tldr", e.extract_by_filename("tldr.bash")?),
                Completion::new_with_data(Shell::Zsh, "tldr", e.extract_by_filename("_tldr")?),
                Completion::new_with_data(Shell::Fish, "tldr", e.extract_by_filename("tldr.fish")?),
            ],
            ..Default::default()
        })
    }
}
