use anyhow::{Result, anyhow};
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppBinary, Completion, DownloadedAssets, ManPage};
use crate::version::AppVersion;

pub struct Dysk {
    client: Arc<GithubClient>,
}

impl Dysk {
    pub const ID: &'static str = "dysk";
    pub const DESCRIPTION: &'static str =
        "Terminal utility to get information on filesystems (df alternative)";
    pub const URL: &'static str = "https://github.com/Canop/dysk";
    const OWNER: &'static str = "Canop";
    const REPO: &'static str = "dysk";
    const EXE_NAME: &'static str = "dysk";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Dysk {
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
            .find(|a| a.starts_with("dysk_") && a.ends_with(".zip"))
            .ok_or_else(|| anyhow!("Can't find dysk zip asset"))?;

        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);

        let binary_data =
            extractor.extract("build/x86_64-unknown-linux-musl/dysk")?;
        let man_data = extractor.extract("build/man/dysk.1")?;
        let bash_data = extractor.extract("build/completion/dysk.bash")?;
        let zsh_data = extractor.extract("build/completion/_dysk")?;
        let fish_data = extractor.extract("build/completion/dysk.fish")?;

        Ok(DownloadedAssets {
            binary: Some(AppBinary::new("dysk", binary_data)),
            man_pages: vec![ManPage::new(1, "dysk.1", man_data)],
            completions: vec![
                Completion::bash("dysk", bash_data),
                Completion::zsh("dysk", zsh_data),
                Completion::fish("dysk", fish_data),
            ],
            ..Default::default()
        })
    }
}
