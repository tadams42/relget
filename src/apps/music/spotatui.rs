use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::apps::gen_completions_subcommand;
use crate::types::{AppBinary, Completion, AppAssets};
use crate::version::AppVersion;

pub struct Spotatui {
    client: Arc<GithubClient>,
}

impl Spotatui {
    pub const ID: &'static str = "spotatui";
    const OWNER: &'static str = "LargeModGames";
    const REPO: &'static str = "spotatui";
    const EXE_NAME: &'static str = "spotatui";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Spotatui {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary:      Some(AppBinary::descriptor(Self::EXE_NAME)),
            completions: vec![Completion::zsh_desc(Self::EXE_NAME), Completion::bash_desc(Self::EXE_NAME), Completion::fish_desc(Self::EXE_NAME)],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release.find_asset(|a| a == "spotatui-linux-x86_64.tar.gz")?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let binary_data = extractor.extract_by_filename("spotatui")?;
        let completions = gen_completions_subcommand("spotatui", &binary_data, "--completions")?;
        Ok(AppAssets {
            binary: Some(AppBinary::new("spotatui", binary_data)),
            completions,
            ..Default::default()
        })
    }
}
