use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppBinary, AppAssets};
use crate::version::AppVersion;

pub struct Difftastic {
    client: Arc<GithubClient>,
}

impl Difftastic {
    pub const ID: &'static str = "difftastic";
    const OWNER: &'static str = "Wilfred";
    const REPO: &'static str = "difftastic";
    const EXE_NAME: &'static str = "difft";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Difftastic {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary:      Some(AppBinary::descriptor("difft")),
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release
            .asset_names()
            .into_iter()
            .find(|a| a == "difft-x86_64-unknown-linux-musl.tar.gz")
            .ok_or_else(|| anyhow!("Can't find difft asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;
        let exe = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "difft")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find difft in archive"))?;
        Ok(AppAssets {
            binary: Some(AppBinary::new("difft", extractor.extract(&exe)?)),
            ..Default::default()
        })
    }
}
