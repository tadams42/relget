use std::sync::Arc;

use anyhow::Result;

use crate::apps::{run_cmd, with_temp_exe};
use crate::{
    App, AppAssets, AppBinary, AppVersion, ArchiveExtractor, Completion, ManPage, RelgetClient,
    Shell,
};

pub struct Rclone {
    client: Arc<dyn RelgetClient>,
}

impl Rclone {
    pub const ID: &'static str = "rclone";
    const OWNER: &'static str = "rclone";
    const REPO: &'static str = "rclone";
    const EXE_NAME: &'static str = "rclone";
    const VERSION_ARG: &'static str = "version";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Rclone {
    fn exe_name(&self) -> &str { Self::EXE_NAME }
    fn cli_version_arg(&self) -> &str { Self::VERSION_ARG }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::new(Self::EXE_NAME)),
            man_pages: vec![ManPage::new(1, "rclone.1")],
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
        let name =
            release.find_asset(|a| a.starts_with("rclone-") && a.ends_with("-linux-amd64.zip"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let binary_data = extractor.extract_by_filename("rclone")?;
        let man_data = extractor.extract_by_filename("rclone.1")?;
        let completions = with_temp_exe("rclone", &binary_data, |exe_path| {
            Ok(vec![
                Completion::new_with_data(
                    Shell::Zsh,
                    "rclone",
                    run_cmd(exe_path, &["completion", "zsh", "-"])?,
                ),
                Completion::new_with_data(
                    Shell::Bash,
                    "rclone",
                    run_cmd(exe_path, &["completion", "bash", "-"])?,
                ),
                Completion::new_with_data(
                    Shell::Fish,
                    "rclone",
                    run_cmd(exe_path, &["completion", "fish", "-"])?,
                ),
            ])
        })?;
        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data("rclone", binary_data)),
            man_pages: vec![ManPage::new_with_data(1, "rclone.1", man_data)],
            completions,
            ..Default::default()
        })
    }
}
