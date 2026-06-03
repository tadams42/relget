use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::apps::{run_cmd, with_temp_exe};
use crate::types::{AppBinary, Completion, AppAssets, ManPage};
use crate::version::AppVersion;

pub struct Xh {
    client: Arc<GithubClient>,
}

impl Xh {
    pub const ID: &'static str = "xh";
    const OWNER: &'static str = "ducaale";
    const REPO: &'static str = "xh";
    const EXE_NAME: &'static str = "xh";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Xh {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary:      Some(AppBinary::descriptor(Self::EXE_NAME)),
            man_pages:   vec![ManPage::descriptor(1, "xh.1")],
            completions: vec![Completion::zsh_desc(Self::EXE_NAME), Completion::bash_desc(Self::EXE_NAME), Completion::fish_desc(Self::EXE_NAME)],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release.find_asset(|a| a.starts_with("xh-") && a.ends_with("-x86_64-unknown-linux-musl.tar.gz"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let binary_data = extractor.extract_by_filename("xh")?;

        let (man_pages, completions) = with_temp_exe("xh", &binary_data, |exe_path| {
            let man_data = run_cmd(exe_path, &["--generate", "man"])?;
            let completions = vec![
                Completion::bash("xh", run_cmd(exe_path, &["--generate", "complete-bash"])?),
                Completion::zsh("xh", run_cmd(exe_path, &["--generate", "complete-zsh"])?),
                Completion::fish("xh", run_cmd(exe_path, &["--generate", "complete-fish"])?),
            ];
            Ok((vec![ManPage::new(1, "xh.1", man_data)], completions))
        })?;

        Ok(AppAssets {
            binary: Some(AppBinary::new("xh", binary_data)),
            man_pages,
            completions,
            ..Default::default()
        })
    }
}
