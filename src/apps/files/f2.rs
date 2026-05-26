use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppBinary, Completion, DownloadedAssets};
use crate::version::AppVersion;

pub struct F2 {
    client: Arc<GithubClient>,
}

impl F2 {
    pub const ID: &'static str = "f2";
    pub const DESCRIPTION: &'static str = "Cross-platform batch file renaming tool";
    pub const URL: &'static str = "https://github.com/ayoisaiah/f2";
    const OWNER: &'static str = "ayoisaiah";
    const REPO: &'static str = "f2";
    const EXE_NAME: &'static str = "f2";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for F2 {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn download(&self) -> Result<DownloadedAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release
            .asset_names()
            .into_iter()
            .find(|a| a.starts_with("f2_") && a.ends_with("_linux_amd64.tar.gz"))
            .ok_or_else(|| anyhow!("Can't find f2 linux amd64 asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;

        let find = |filename: &str| -> Result<String> {
            members
                .iter()
                .find(|m| {
                    Path::new(m)
                        .file_name()
                        .map(|f| f == filename)
                        .unwrap_or(false)
                })
                .cloned()
                .ok_or_else(|| anyhow!("Can't find {} in archive", filename))
        };

        Ok(DownloadedAssets {
            binary: Some(AppBinary::new("f2", extractor.extract(&find("f2")?)?)),
            completions: vec![
                Completion::zsh("f2", extractor.extract(&find("f2.zsh")?)?),
                Completion::bash("f2", extractor.extract(&find("f2.bash")?)?),
                Completion::fish("f2", extractor.extract(&find("f2.fish")?)?),
            ],
            ..Default::default()
        })
    }
}
