use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppBinary, Completion, AppAssets};
use crate::version::AppVersion;

pub struct GoJq {
    client: Arc<GithubClient>,
}

impl GoJq {
    pub const ID: &'static str = "gojq";
    pub const CATEGORY: &'static str = "data";
    pub const DESCRIPTION: &'static str = "Pure Go implementation of jq with extended features";
    pub const URL: &'static str = "https://github.com/itchyny/gojq";
    const OWNER: &'static str = "itchyny";
    const REPO: &'static str = "gojq";
    const EXE_NAME: &'static str = "gojq";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for GoJq {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary:      Some(AppBinary::descriptor("gojq")),
            completions: vec![Completion::zsh_desc("gojq")],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release
            .asset_names()
            .into_iter()
            .find(|a| a.starts_with("gojq_") && a.ends_with("_linux_amd64.tar.gz"))
            .ok_or_else(|| anyhow!("Can't find gojq asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;

        let exe = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "gojq")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find gojq in archive"))?;
        let zsh = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "_gojq")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find _gojq zsh completion in archive"))?;

        Ok(AppAssets {
            binary: Some(AppBinary::new("gojq", extractor.extract(&exe)?)),
            // Only zsh completion is packaged (no runtime generation supported)
            completions: vec![Completion::zsh("gojq", extractor.extract(&zsh)?)],
            ..Default::default()
        })
    }
}
