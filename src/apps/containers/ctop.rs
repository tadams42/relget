use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::clients::GithubClient;
use crate::types::{AppAssets, AppBinary};
use crate::version::AppVersion;

pub struct Ctop {
    client: Arc<GithubClient>,
}

impl Ctop {
    pub const ID: &'static str = "ctop";
    const OWNER: &'static str = "bcicen";
    const REPO: &'static str = "ctop";
    const EXE_NAME: &'static str = "ctop";

    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Ctop {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn cli_version_arg(&self) -> &str { "-v" }

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
        let name = release.find_asset(|a| a.starts_with("ctop-") && a.ends_with("-linux-amd64"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        Ok(AppAssets {
            binary: Some(AppBinary::new(Self::EXE_NAME, asset.data)),
            ..Default::default()
        })
    }
}
