use std::sync::Arc;

use anyhow::Result;

use crate::apps::gen_completions_shell_flag;
use crate::{
    App, AppAssets, AppBinary, AppVersion, ArchiveExtractor, Completion, RelgetClient, Shell,
};

pub struct Fnm {
    client: Arc<dyn RelgetClient>,
}

impl Fnm {
    pub const ID: &'static str = "fnm";
    const OWNER: &'static str = "Schniz";
    const REPO: &'static str = "fnm";
    const EXE_NAME: &'static str = "fnm";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Fnm {
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
        let name = release.find_asset(|a| a == "fnm-linux.zip")?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let binary_data = extractor.extract_by_filename("fnm")?;
        let completions =
            gen_completions_shell_flag("fnm", &binary_data, "completions", "--shell")?;
        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data("fnm", binary_data)),
            completions,
            ..Default::default()
        })
    }
}
