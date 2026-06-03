use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::installer::gen_completions_subcommand;
use crate::types::{AppBinary, Completion, AppAssets};
use crate::version::AppVersion;

pub struct Spotatui {
    client: Arc<GithubClient>,
}

impl Spotatui {
    pub const ID: &'static str = "spotatui";
    pub const DESCRIPTION: &'static str = "Terminal UI for Spotify";
    pub const URL: &'static str = "https://github.com/LargeModGames/spotatui";
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
            binary:      Some(AppBinary::descriptor("spotatui")),
            completions: vec![Completion::zsh_desc("spotatui"), Completion::bash_desc("spotatui"), Completion::fish_desc("spotatui")],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release
            .asset_names()
            .into_iter()
            .find(|a| a == "spotatui-linux-x86_64.tar.gz")
            .ok_or_else(|| anyhow!("Can't find spotatui-linux-x86_64.tar.gz"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;
        let exe = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "spotatui")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find spotatui in archive"))?;
        let binary_data = extractor.extract(&exe)?;
        let completions = gen_completions_subcommand("spotatui", &binary_data, "--completions")?;
        Ok(AppAssets {
            binary: Some(AppBinary::new("spotatui", binary_data)),
            completions,
            ..Default::default()
        })
    }
}
