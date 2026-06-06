use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppAssets, AppBinary, Completion, ManPage};
use crate::version::AppVersion;

pub struct JsonGrep {
    client: Arc<GithubClient>,
}

impl JsonGrep {
    pub const ID: &'static str = "jsongrep";
    const OWNER: &'static str = "micahkepe";
    const REPO: &'static str = "jsongrep";
    const EXE_NAME: &'static str = "jg";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for JsonGrep {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::descriptor(Self::EXE_NAME)),
            man_pages: vec![ManPage::descriptor(1, "jg.1")],
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
        let name = release.find_asset(|a| {
            a.contains("x86_64-unknown-linux-musl") && a.ends_with(".tar.gz")
        })?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        Ok(AppAssets {
            binary: Some(AppBinary::new(
                Self::EXE_NAME,
                extractor.extract_by_filename(Self::EXE_NAME)?,
            )),
            man_pages: vec![ManPage::new(1, "jg.1", extractor.extract_by_filename("jg.1")?)],
            completions: vec![
                Completion::zsh(Self::EXE_NAME, extractor.extract_by_filename("jg.zsh")?),
                Completion::bash(Self::EXE_NAME, extractor.extract_by_filename("jg.bash")?),
                Completion::fish(Self::EXE_NAME, extractor.extract_by_filename("jg.fish")?),
            ],
            ..Default::default()
        })
    }
}
