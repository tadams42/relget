use anyhow::{Result, anyhow};
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppBinary, AppAssets};
use crate::version::AppVersion;

pub struct RustAnalyzer {
    client: Arc<GithubClient>,
}

impl RustAnalyzer {
    pub const ID: &'static str = "rust-analyzer";
    const OWNER: &'static str = "rust-lang";
    const REPO: &'static str = "rust-analyzer";
    const EXE_NAME: &'static str = "rust-analyzer";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for RustAnalyzer {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        // GitHub tags use CalVer ("2026-05-18") but the binary reports a build-counter
        // version ("0.3.2904"). The release body always contains the latter on line 2 as
        // "(v0.3.2904)", so we scan the body first to get a version comparable to the
        // installed one, falling back to the tag if the body format ever changes.
        if let Some(body) = release.data["body"].as_str() {
            if let Some(v) = AppVersion::find_in(body) {
                return Ok(v);
            }
        }
        release.version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary:      Some(AppBinary::descriptor(Self::EXE_NAME)),
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release
            .asset_names()
            .into_iter()
            .find(|a| a == "rust-analyzer-x86_64-unknown-linux-gnu.gz")
            .ok_or_else(|| anyhow!("Can't find rust-analyzer asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        // Single .gz file decompresses to one file; member name = name without .gz
        let member = "rust-analyzer-x86_64-unknown-linux-gnu";
        let binary_data = extractor.extract(member)?;
        Ok(AppAssets {
            binary: Some(AppBinary::new("rust-analyzer", binary_data)),
            ..Default::default()
        })
    }
}
