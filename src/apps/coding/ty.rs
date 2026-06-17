use anyhow::Result;
use std::sync::Arc;

use crate::apps::{App, gen_completions_subcommand, with_temp_exe};
use crate::archive::ArchiveExtractor;
use crate::clients::RelgetClient;
use crate::types::{AppAssets, AppBinary, Completion};
use crate::version::AppVersion;

pub struct Ty {
    client: Arc<dyn RelgetClient>,
}

impl Ty {
    pub const ID: &'static str = "ty";
    const OWNER: &'static str = "astral-sh";
    const REPO: &'static str = "ty";
    const EXE_NAME: &'static str = "ty";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Ty {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::descriptor(Self::EXE_NAME)),
            completions: vec![
                Completion::zsh_desc(Self::EXE_NAME),
                Completion::bash_desc(Self::EXE_NAME),
                Completion::fish_desc(Self::EXE_NAME),
            ],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release.find_asset(|a| a == "ty-x86_64-unknown-linux-musl.tar.gz")?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let binary_data = extractor.extract_by_filename("ty")?;

        let completions = with_temp_exe("ty", &binary_data, |_| {
            gen_completions_subcommand("ty", &binary_data, "generate-shell-completion")
        })?;

        Ok(AppAssets {
            binary: Some(AppBinary::new("ty", binary_data)),
            completions,
            ..Default::default()
        })
    }
}
