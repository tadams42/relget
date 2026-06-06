use anyhow::Result;
use std::sync::Arc;

use crate::apps::{App, gen_completions_with_shell_arg};
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppAssets, AppBinary, Completion};
use crate::version::AppVersion;

pub struct Erdtree {
    client: Arc<GithubClient>,
}

impl Erdtree {
    pub const ID: &'static str = "erdtree";
    const OWNER: &'static str = "solidiquis";
    const REPO: &'static str = "erdtree";
    const EXE_NAME: &'static str = "erd";

    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Erdtree {
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
        let name = release.find_asset(|a| {
            a.starts_with("erd-") && a.ends_with("-x86_64-unknown-linux-musl.tar.gz")
        })?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let binary_data = extractor.extract_by_filename(Self::EXE_NAME)?;
        let completions =
            gen_completions_with_shell_arg(Self::EXE_NAME, &binary_data, &["--completions"])?;
        Ok(AppAssets {
            binary: Some(AppBinary::new(Self::EXE_NAME, binary_data)),
            completions,
            ..Default::default()
        })
    }
}
