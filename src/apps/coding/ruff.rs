use std::sync::Arc;

use anyhow::Result;

use crate::apps::{gen_completions_subcommand, with_temp_exe};
use crate::{
    App, AppAssets, AppBinary, AppVersion, ArchiveExtractor, Completion, RelgetClient, Shell,
};

pub struct Ruff {
    client: Arc<dyn RelgetClient>,
}

impl Ruff {
    pub const ID: &'static str = "ruff";
    const OWNER: &'static str = "astral-sh";
    const REPO: &'static str = "ruff";
    const EXE_NAME: &'static str = "ruff";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Ruff {
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
        let name = release.find_asset(|a| a == "ruff-x86_64-unknown-linux-musl.tar.gz")?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let binary_data = extractor.extract_by_filename("ruff")?;

        let completions = with_temp_exe("ruff", &binary_data, |_| {
            gen_completions_subcommand("ruff", &binary_data, "generate-shell-completion")
        })?;

        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data("ruff", binary_data)),
            completions,
            ..Default::default()
        })
    }
}
