use std::sync::Arc;

use anyhow::Result;

use crate::apps::gen_completions_subcommand;
use crate::{
    App, AppAssets, AppBinary, AppVersion, ArchiveExtractor, Completion, RelgetClient, Shell,
};

pub struct Xq {
    client: Arc<dyn RelgetClient>,
}

impl Xq {
    pub const ID: &'static str = "xq";
    const OWNER: &'static str = "sibprogrammer";
    const REPO: &'static str = "xq";
    const EXE_NAME: &'static str = "xq";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Xq {
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
        let name =
            release.find_asset(|a| a.starts_with("xq_") && a.ends_with("_linux_amd64.tar.gz"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let binary_data = extractor.extract_by_filename("xq")?;
        let completions = gen_completions_subcommand("xq", &binary_data, "completion")?;
        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data("xq", binary_data)),
            completions,
            ..Default::default()
        })
    }
}
