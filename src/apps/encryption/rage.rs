use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppAssets, AppBinary};
use crate::version::AppVersion;

pub struct Rage {
    client: Arc<GithubClient>,
}

impl Rage {
    pub const ID: &'static str = "rage";
    const OWNER: &'static str = "str4d";
    const REPO: &'static str = "rage";
    const EXE_NAME: &'static str = "rage";

    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Rage {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::descriptor(Self::EXE_NAME)),
            other_bins: vec![
                AppBinary::descriptor("rage-keygen"),
                AppBinary::descriptor("rage-mount"),
            ],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release
            .find_asset(|a| a.starts_with("rage-") && a.ends_with("-x86_64-linux.tar.gz"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        Ok(AppAssets {
            binary: Some(AppBinary::new(
                Self::EXE_NAME,
                extractor.extract_by_filename(Self::EXE_NAME)?,
            )),
            other_bins: vec![
                AppBinary::new("rage-keygen", extractor.extract_by_filename("rage-keygen")?),
                AppBinary::new("rage-mount", extractor.extract_by_filename("rage-mount")?),
            ],
            ..Default::default()
        })
    }
}
