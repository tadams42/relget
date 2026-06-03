use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::installer::gen_completions_subcommand;
use crate::types::{AppBinary, Completion, AppAssets};
use crate::version::AppVersion;

pub struct Restish {
    client: Arc<GithubClient>,
}

impl Restish {
    pub const ID: &'static str = "restish";
    const OWNER: &'static str = "rest-sh";
    const REPO: &'static str = "restish";
    const EXE_NAME: &'static str = "restish";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Restish {
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
        let name = release
            .asset_names()
            .into_iter()
            .find(|a| a.starts_with("restish-") && a.ends_with("-linux-amd64.tar.gz"))
            .ok_or_else(|| anyhow!("Can't find restish asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;
        let exe = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "restish")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find restish in archive"))?;
        let binary_data = extractor.extract(&exe)?;
        let completions = gen_completions_subcommand("restish", &binary_data, "completion")?;
        Ok(AppAssets {
            binary: Some(AppBinary::new("restish", binary_data)),
            completions,
            ..Default::default()
        })
    }
}
