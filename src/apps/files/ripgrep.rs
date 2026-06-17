use anyhow::Result;
use std::sync::Arc;

use crate::apps::{App, run_cmd, with_temp_exe};
use crate::archive::ArchiveExtractor;
use crate::clients::RelgetClient;
use crate::types::{AppAssets, AppBinary, Completion, ManPage};
use crate::version::AppVersion;

pub struct Ripgrep {
    client: Arc<dyn RelgetClient>,
}

impl Ripgrep {
    pub const ID: &'static str = "ripgrep";
    const OWNER: &'static str = "BurntSushi";
    const REPO: &'static str = "ripgrep";
    const EXE_NAME: &'static str = "rg";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Ripgrep {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::descriptor(Self::EXE_NAME)),
            man_pages: vec![ManPage::descriptor(1, "rg.1")],
            completions: vec![
                Completion::zsh_desc(Self::EXE_NAME),
                Completion::bash_desc(Self::EXE_NAME),
                Completion::fish_desc(Self::EXE_NAME),
            ],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release.find_asset(|a| {
            a.starts_with("ripgrep-") && a.ends_with("-x86_64-unknown-linux-musl.tar.gz")
        })?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let binary_data = extractor.extract_by_filename("rg")?;
        let man_data = extractor.extract_by_filename("rg.1")?;
        let completions = with_temp_exe("rg", &binary_data, |exe| {
            Ok(vec![
                Completion::zsh("rg", run_cmd(exe, &["--generate", "complete-zsh"])?),
                Completion::bash("rg", run_cmd(exe, &["--generate", "complete-bash"])?),
                Completion::fish("rg", run_cmd(exe, &["--generate", "complete-fish"])?),
            ])
        })?;
        Ok(AppAssets {
            binary: Some(AppBinary::new("rg", binary_data)),
            man_pages: vec![ManPage::new(1, "rg.1", man_data)],
            completions,
            ..Default::default()
        })
    }
}
