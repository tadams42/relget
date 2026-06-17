use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::RelgetClient;
use crate::types::{AppAssets, AppBinary, Completion};
use crate::version::AppVersion;

pub struct GoJq {
    client: Arc<dyn RelgetClient>,
}

impl GoJq {
    pub const ID: &'static str = "gojq";
    const OWNER: &'static str = "itchyny";
    const REPO: &'static str = "gojq";
    const EXE_NAME: &'static str = "gojq";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
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
            binary: Some(AppBinary::descriptor(Self::EXE_NAME)),
            completions: vec![Completion::zsh_desc(Self::EXE_NAME)],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name =
            release.find_asset(|a| a.starts_with("gojq_") && a.ends_with("_linux_amd64.tar.gz"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        Ok(AppAssets {
            binary: Some(AppBinary::new("gojq", extractor.extract_by_filename("gojq")?)),
            // Only zsh completion is packaged (no runtime generation supported)
            completions: vec![Completion::zsh(
                "gojq",
                extractor.extract_by_filename("_gojq")?,
            )],
            ..Default::default()
        })
    }
}
