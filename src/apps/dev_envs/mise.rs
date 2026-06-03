use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::installer::gen_completions_subcommand;
use crate::types::{AppBinary, Completion, AppAssets, ManPage};
use crate::version::AppVersion;

pub struct Mise {
    client: Arc<GithubClient>,
}

impl Mise {
    pub const ID: &'static str = "mise";
    const OWNER: &'static str = "jdx";
    const REPO: &'static str = "mise";
    const EXE_NAME: &'static str = "mise";
    const VERSION_ARG: &'static str = "version";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Mise {
    fn exe_name(&self) -> &str { Self::EXE_NAME }
    fn cli_version_arg(&self) -> &str { Self::VERSION_ARG }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary:      Some(AppBinary::descriptor("mise")),
            man_pages:   vec![ManPage::descriptor(1, "mise.1")],
            completions: vec![Completion::zsh_desc("mise"), Completion::bash_desc("mise"), Completion::fish_desc("mise")],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release
            .asset_names()
            .into_iter()
            .find(|a| a.starts_with("mise-") && a.ends_with("-linux-x64.tar.gz"))
            .ok_or_else(|| anyhow!("Can't find mise asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;

        let exe = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "mise")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find mise in archive"))?;
        let man = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "mise.1")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find mise.1 in archive"))?;

        let binary_data = extractor.extract(&exe)?;
        let man_data = extractor.extract(&man)?;
        let completions = gen_completions_subcommand("mise", &binary_data, "completion")?;

        Ok(AppAssets {
            binary: Some(AppBinary::new("mise", binary_data)),
            man_pages: vec![ManPage::new(1, "mise.1", man_data)],
            completions,
            ..Default::default()
        })
    }
}
