use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppAssets, AppBinary, Completion};
use crate::version::AppVersion;

pub struct F2 {
    client: Arc<GithubClient>,
}

impl F2 {
    pub const ID: &'static str = "f2";
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
        let name =
            release.find_asset(|a| a.starts_with("f2_") && a.ends_with("_linux_amd64.tar.gz"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let e = ArchiveExtractor::new(&name, asset.data);
        Ok(AppAssets {
            binary: Some(AppBinary::new("f2", e.extract_by_filename("f2")?)),
            completions: vec![
                Completion::zsh("f2", e.extract_by_filename("f2.zsh")?),
                Completion::bash("f2", e.extract_by_filename("f2.bash")?),
                Completion::fish("f2", e.extract_by_filename("f2.fish")?),
            ],
            ..Default::default()
        })
    }
}
