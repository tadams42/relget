use std::sync::Arc;

use anyhow::Result;

use crate::apps::gen_completions_subcommand;
use crate::{
    App, AppAssets, AppBinary, AppVersion, ArchiveExtractor, Completion, ManPage, RelgetClient,
    Shell,
};

pub struct Bat {
    client: Arc<dyn RelgetClient>,
}

impl Bat {
    pub const ID: &'static str = "bat";
    const OWNER: &'static str = "sharkdp";
    const REPO: &'static str = "bat";
    const EXE_NAME: &'static str = "bat";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Bat {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::new(Self::EXE_NAME)),
            man_pages: vec![ManPage::new(1, "bat.1")],
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
            a.starts_with("bat-") && a.ends_with("-x86_64-unknown-linux-musl.tar.gz")
        })?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let binary_data = extractor.extract_by_filename("bat")?;
        let man_data = extractor.extract_by_filename("bat.1")?;
        let completions = gen_completions_subcommand("bat", &binary_data, "--completion")?;
        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data("bat", binary_data)),
            man_pages: vec![ManPage::new_with_data(1, "bat.1", man_data)],
            completions,
            ..Default::default()
        })
    }
}
