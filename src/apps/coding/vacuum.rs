use anyhow::Result;
use std::sync::Arc;

use crate::apps::{App, gen_completions_subcommand};
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppAssets, AppBinary, Completion};
use crate::version::AppVersion;

pub struct Vacuum {
    client: Arc<GithubClient>,
}

impl Vacuum {
    pub const ID: &'static str = "vacuum";
    const OWNER: &'static str = "daveshanley";
    const REPO: &'static str = "vacuum";
    const EXE_NAME: &'static str = "vacuum";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Vacuum {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn cli_version_arg(&self) -> &str { "version" }

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
        let name = release.find_asset(|a| a.contains("linux_x86_64") && a.ends_with(".tar.gz"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let binary_data = extractor.extract_by_filename(Self::EXE_NAME)?;
        let completions = gen_completions_subcommand("vacuum", &binary_data, "completion")?;
        Ok(AppAssets {
            binary: Some(AppBinary::new(Self::EXE_NAME, binary_data)),
            completions,
            ..Default::default()
        })
    }
}
