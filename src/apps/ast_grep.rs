use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::github::GithubClient;
use crate::installer::gen_completions_subcommand;
use crate::types::{AppBinary, DownloadedAssets};
use crate::version::AppVersion;

pub struct AstGrep {
    client: Arc<GithubClient>,
}

impl AstGrep {
    const OWNER: &'static str = "ast-grep";
    const REPO: &'static str = "ast-grep";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for AstGrep {
    fn exe_name(&self) -> &str { "ast-grep" }
    fn url(&self) -> &str { "https://github.com/ast-grep/ast-grep" }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn download(&self) -> Result<DownloadedAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release
            .asset_names()
            .into_iter()
            .find(|a| a == "app-x86_64-unknown-linux-gnu.zip")
            .ok_or_else(|| anyhow!("Can't find ast-grep asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;

        let ag_entry = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "ast-grep")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find ast-grep in archive"))?;
        let sg_entry = members
            .iter()
            .find(|m| Path::new(m).file_name().map(|f| f == "sg").unwrap_or(false))
            .cloned()
            .ok_or_else(|| anyhow!("Can't find sg in archive"))?;

        let ag_data = extractor.extract(&ag_entry)?;
        let sg_data = extractor.extract(&sg_entry)?;

        let mut completions = gen_completions_subcommand("ast-grep", &ag_data, "completions")?;
        completions.extend(gen_completions_subcommand("sg", &sg_data, "completions")?);

        Ok(DownloadedAssets {
            binary: Some(AppBinary::new("ast-grep", ag_data)),
            other_bins: vec![AppBinary::new("sg", sg_data)],
            completions,
            ..Default::default()
        })
    }
}
