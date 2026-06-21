use std::sync::Arc;

use anyhow::Result;

use crate::apps::gen_completions_subcommand;
use crate::{
    App, AppAssets, AppBinary, AppVersion, ArchiveExtractor, Completion, RelgetClient, Shell,
};

pub struct Gitleaks {
    client: Arc<dyn RelgetClient>,
}

impl Gitleaks {
    pub const ID: &'static str = "gitleaks";
    const OWNER: &'static str = "gitleaks";
    const REPO: &'static str = "gitleaks";
    const EXE_NAME: &'static str = "gitleaks";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Gitleaks {
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
        let name = release
            .find_asset(|a| a.starts_with("gitleaks_") && a.ends_with("_linux_x64.tar.gz"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let binary_data = extractor.extract_by_filename("gitleaks")?;
        let completions = gen_completions_subcommand("gitleaks", &binary_data, "completion")?;
        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data("gitleaks", binary_data)),
            completions,
            ..Default::default()
        })
    }
}
