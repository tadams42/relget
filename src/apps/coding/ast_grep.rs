use anyhow::Result;
use std::sync::Arc;

use crate::apps::{App, gen_completions_subcommand};
use crate::archive::ArchiveExtractor;
use crate::clients::RelgetClient;
use crate::types::{AppAssets, AppBinary, Completion};
use crate::version::AppVersion;

pub struct AstGrep {
    client: Arc<dyn RelgetClient>,
}

impl AstGrep {
    pub const ID: &'static str = "ast-grep";
    const OWNER: &'static str = "ast-grep";
    const REPO: &'static str = "ast-grep";
    const EXE_NAME: &'static str = "ast-grep";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for AstGrep {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::descriptor(Self::EXE_NAME)),
            other_bins: vec![AppBinary::descriptor("sg")],
            completions: vec![
                Completion::zsh_desc(Self::EXE_NAME),
                Completion::bash_desc(Self::EXE_NAME),
                Completion::fish_desc(Self::EXE_NAME),
                Completion::zsh_desc("sg"),
                Completion::bash_desc("sg"),
                Completion::fish_desc("sg"),
            ],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release.find_asset(|a| a == "app-x86_64-unknown-linux-gnu.zip")?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let ag_data = extractor.extract_by_filename("ast-grep")?;
        let sg_data = extractor.extract_by_filename("sg")?;
        let mut completions = gen_completions_subcommand("ast-grep", &ag_data, "completions")?;
        completions.extend(gen_completions_subcommand("sg", &sg_data, "completions")?);
        Ok(AppAssets {
            binary: Some(AppBinary::new("ast-grep", ag_data)),
            other_bins: vec![AppBinary::new("sg", sg_data)],
            completions,
            ..Default::default()
        })
    }
}
