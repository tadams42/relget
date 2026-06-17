use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::apps::app_assets::{AppAssets, AppBinary, Completion, ManPage, Shell};
use crate::archive::ArchiveExtractor;
use crate::clients::RelgetClient;
use crate::version::AppVersion;

pub struct JsonGrep {
    client: Arc<dyn RelgetClient>,
}

impl JsonGrep {
    pub const ID: &'static str = "jsongrep";
    const OWNER: &'static str = "micahkepe";
    const REPO: &'static str = "jsongrep";
    const EXE_NAME: &'static str = "jg";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for JsonGrep {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::new(Self::EXE_NAME)),
            man_pages: vec![ManPage::new(1, "jg.1")],
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
        let name = release
            .find_asset(|a| a.contains("x86_64-unknown-linux-musl") && a.ends_with(".tar.gz"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data(
                Self::EXE_NAME,
                extractor.extract_by_filename(Self::EXE_NAME)?,
            )),
            man_pages: vec![ManPage::new_with_data(
                1,
                "jg.1",
                extractor.extract_by_filename("jg.1")?,
            )],
            completions: vec![
                Completion::new_with_data(
                    Shell::Zsh,
                    Self::EXE_NAME,
                    extractor.extract_by_filename("jg.zsh")?,
                ),
                Completion::new_with_data(
                    Shell::Bash,
                    Self::EXE_NAME,
                    extractor.extract_by_filename("jg.bash")?,
                ),
                Completion::new_with_data(
                    Shell::Fish,
                    Self::EXE_NAME,
                    extractor.extract_by_filename("jg.fish")?,
                ),
            ],
            ..Default::default()
        })
    }
}
