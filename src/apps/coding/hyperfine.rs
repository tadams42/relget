use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppAssets, AppBinary, Completion, ManPage};
use crate::version::AppVersion;

pub struct Hyperfine {
    client: Arc<GithubClient>,
}

impl Hyperfine {
    pub const ID: &'static str = "hyperfine";
    const OWNER: &'static str = "sharkdp";
    const REPO: &'static str = "hyperfine";
    const EXE_NAME: &'static str = "hyperfine";

    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Hyperfine {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::descriptor(Self::EXE_NAME)),
            man_pages: vec![ManPage::descriptor(1, "hyperfine.1")],
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
            a.starts_with("hyperfine-") && a.ends_with("-x86_64-unknown-linux-musl.tar.gz")
        })?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        Ok(AppAssets {
            binary: Some(AppBinary::new(
                Self::EXE_NAME,
                extractor.extract_by_filename(Self::EXE_NAME)?,
            )),
            man_pages: vec![ManPage::new(
                1,
                "hyperfine.1",
                extractor.extract_by_filename("hyperfine.1")?,
            )],
            completions: vec![
                Completion::bash(Self::EXE_NAME, extractor.extract_by_filename("hyperfine.bash")?),
                Completion::zsh(Self::EXE_NAME, extractor.extract_by_filename("_hyperfine")?),
                Completion::fish(Self::EXE_NAME, extractor.extract_by_filename("hyperfine.fish")?),
            ],
            ..Default::default()
        })
    }
}
