use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::RelgetClient;
use crate::types::{AppAssets, AppBinary};
use crate::version::AppVersion;

pub struct Lazygit {
    client: Arc<dyn RelgetClient>,
}

impl Lazygit {
    pub const ID: &'static str = "lazygit";
    const OWNER: &'static str = "jesseduffield";
    const REPO: &'static str = "lazygit";
    const EXE_NAME: &'static str = "lazygit";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Lazygit {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::descriptor(Self::EXE_NAME)),
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release.find_asset(|a| {
            a.starts_with("lazygit_")
                && (a.ends_with("_Linux_x86_64.tar.gz") || a.ends_with("_linux_x86_64.tar.gz"))
        })?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        Ok(AppAssets {
            binary: Some(AppBinary::new("lazygit", extractor.extract_by_filename("lazygit")?)),
            ..Default::default()
        })
    }
}
