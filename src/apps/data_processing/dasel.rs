use anyhow::Result;
use std::sync::Arc;

use crate::apps::{App, gen_completions_subcommand, run_cmd, with_temp_exe};
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppAssets, AppBinary, Completion, ManPage};
use crate::version::AppVersion;

pub struct Dasel {
    client: Arc<GithubClient>,
}

impl Dasel {
    pub const ID: &'static str = "dasel";
    const OWNER: &'static str = "TomWright";
    const REPO: &'static str = "dasel";
    const EXE_NAME: &'static str = "dasel";
    const VERSION_ARG: &'static str = "version";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Dasel {
    fn exe_name(&self) -> &str { Self::EXE_NAME }
    fn cli_version_arg(&self) -> &str { Self::VERSION_ARG }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::descriptor(Self::EXE_NAME)),
            man_pages: vec![ManPage::descriptor(1, "dasel.1")],
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
        let name = release.find_asset(|a| a == "dasel_linux_amd64.gz")?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let binary_data = extractor.extract_by_filename("dasel_linux_amd64")?;

        let (completions, man_pages) = with_temp_exe("dasel", &binary_data, |exe_path| {
            let completions = gen_completions_subcommand("dasel", &binary_data, "completion")?;
            let man_data = run_cmd(exe_path, &["man"])?;
            Ok((completions, vec![ManPage::new(1, "dasel.1", man_data)]))
        })?;

        Ok(AppAssets {
            binary: Some(AppBinary::new("dasel", binary_data)),
            completions,
            man_pages,
            ..Default::default()
        })
    }
}
