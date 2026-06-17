use anyhow::Result;
use std::sync::Arc;

use crate::apps::{App, gen_completions_with_shell_arg};
use crate::archive::ArchiveExtractor;
use crate::clients::RelgetClient;
use crate::types::{AppAssets, AppBinary, Completion};
use crate::version::AppVersion;

pub struct Hl {
    client: Arc<dyn RelgetClient>,
}

impl Hl {
    pub const ID: &'static str = "hl";
    const OWNER: &'static str = "pamburus";
    const REPO: &'static str = "hl";
    const EXE_NAME: &'static str = "hl";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Hl {
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
        let name = release.find_asset(|a| a == "hl-linux-x86_64-musl.tar.gz")?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let binary_data = extractor.extract_by_filename(Self::EXE_NAME)?;
        let completions =
            gen_completions_with_shell_arg("hl", &binary_data, &["--shell-completions"])?;
        Ok(AppAssets {
            binary: Some(AppBinary::new(Self::EXE_NAME, binary_data)),
            completions,
            ..Default::default()
        })
    }
}
