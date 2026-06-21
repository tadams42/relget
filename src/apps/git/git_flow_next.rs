use std::sync::Arc;

use anyhow::Result;

use crate::apps::gen_completions_subcommand;
use crate::{
    App, AppAssets, AppBinary, AppVersion, ArchiveExtractor, Completion, RelgetClient, Shell,
};

pub struct GitFlowNext {
    client: Arc<dyn RelgetClient>,
}

impl GitFlowNext {
    pub const ID: &'static str = "git-flow-next";
    const OWNER: &'static str = "gittower";
    const REPO: &'static str = "git-flow-next";
    const EXE_NAME: &'static str = "git-flow";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for GitFlowNext {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn cli_version_arg(&self) -> &str { "version" }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::new(Self::EXE_NAME)),
            completions: vec![
                Completion::new(Shell::Zsh, Self::EXE_NAME),
                Completion::new(Shell::Bash, Self::EXE_NAME),
                Completion::new(Shell::Fish, Self::EXE_NAME),
            ],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release.find_asset(|a| {
            a.starts_with("git-flow-next-") && a.ends_with("-linux-amd64.tar.gz")
        })?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;
        let binary_data = extractor.extract(&members[0])?;
        let completions = gen_completions_subcommand("git-flow", &binary_data, "completion")?;
        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data(Self::EXE_NAME, binary_data)),
            completions,
            ..Default::default()
        })
    }
}
