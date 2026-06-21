use std::sync::Arc;

use anyhow::Result;

use crate::apps::gen_completions_subcommand;
use crate::{
    App, AppAssets, AppBinary, AppVersion, ArchiveExtractor, Completion, ManPage, RelgetClient,
    Shell,
};

pub struct Deadbranch {
    client: Arc<dyn RelgetClient>,
}

impl Deadbranch {
    pub const ID: &'static str = "deadbranch";
    const OWNER: &'static str = "armgabrielyan";
    const REPO: &'static str = "deadbranch";
    const EXE_NAME: &'static str = "deadbranch";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Deadbranch {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::new(Self::EXE_NAME)),
            man_pages: vec![ManPage::new(1, "deadbranch.1")],
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
        let binary_data = extractor.extract_by_filename("deadbranch")?;
        let completions = gen_completions_subcommand(Self::EXE_NAME, &binary_data, "completions")?;
        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data(Self::EXE_NAME, binary_data)),
            man_pages: vec![ManPage::new_with_data(
                1,
                "deadbranch.1",
                extractor.extract_by_filename("deadbranch.1")?,
            )],
            completions,
            ..Default::default()
        })
    }
}
