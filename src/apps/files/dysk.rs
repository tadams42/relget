use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppAssets, AppBinary, Completion, ManPage};
use crate::version::AppVersion;

pub struct Dysk {
    client: Arc<GithubClient>,
}

impl Dysk {
    pub const ID: &'static str = "dysk";
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

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::descriptor(Self::EXE_NAME)),
            man_pages: vec![ManPage::descriptor(1, "dysk.1")],
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
        let name = release.find_asset(|a| a.starts_with("dysk_") && a.ends_with(".zip"))?;

        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);

        let binary_data = extractor.extract("build/x86_64-unknown-linux-musl/dysk")?;
        let man_data = extractor.extract("build/man/dysk.1")?;
        let bash_data = extractor.extract("build/completion/dysk.bash")?;
        let zsh_data = extractor.extract("build/completion/_dysk")?;
        let fish_data = extractor.extract("build/completion/dysk.fish")?;

        Ok(AppAssets {
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
