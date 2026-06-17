use anyhow::Result;
use std::sync::Arc;

use crate::apps::app_assets::{AppAssets, AppBinary, Completion, Shell};
use crate::apps::{App, gen_completions_subcommand};
use crate::archive::ArchiveExtractor;
use crate::clients::RelgetClient;
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
            binary: Some(AppBinary::new(Self::EXE_NAME)),
            other_bins: vec![AppBinary::new("sg")],
            completions: vec![
                Completion::new(Shell::Zsh, Self::EXE_NAME),
                Completion::new(Shell::Bash, Self::EXE_NAME),
                Completion::new(Shell::Fish, Self::EXE_NAME),
                Completion::new(Shell::Zsh, "sg"),
                Completion::new(Shell::Bash, "sg"),
                Completion::new(Shell::Fish, "sg"),
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
            binary: Some(AppBinary::new_with_data("ast-grep", ag_data)),
            other_bins: vec![AppBinary::new_with_data("sg", sg_data)],
            completions,
            ..Default::default()
        })
    }
}
