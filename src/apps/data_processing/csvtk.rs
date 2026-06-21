use std::sync::Arc;

use anyhow::Result;

use crate::apps::gen_completions_shell_flag;
use crate::{
    App, AppAssets, AppBinary, AppVersion, ArchiveExtractor, Completion, RelgetClient, Shell,
};

pub struct Csvtk {
    client: Arc<dyn RelgetClient>,
}

impl Csvtk {
    pub const ID: &'static str = "csvtk";
    const OWNER: &'static str = "shenwei356";
    const REPO: &'static str = "csvtk";
    const EXE_NAME: &'static str = "csvtk";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Csvtk {
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
        let name = release.find_asset(|a| a == "csvtk_linux_amd64.tar.gz")?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let binary_data = extractor.extract_by_filename("csvtk")?;
        let completions =
            gen_completions_shell_flag("csvtk", &binary_data, "genautocomplete", "--shell")?;
        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data("csvtk", binary_data)),
            completions,
            ..Default::default()
        })
    }
}
