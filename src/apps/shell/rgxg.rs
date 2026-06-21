use std::sync::Arc;

use anyhow::Result;

use crate::{
    App, AppAssets, AppBinary, AppVersion, ArchiveExtractor, Completion, ManPage, RelgetClient,
    Shell,
};

pub struct Rgxg {
    client: Arc<dyn RelgetClient>,
}

impl Rgxg {
    pub const ID: &'static str = "rgxg";
    const OWNER: &'static str = "tadams42";
    const REPO: &'static str = "rgxg";
    const EXE_NAME: &'static str = "rgxg";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Rgxg {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn cli_version_arg(&self) -> &str { "-v" }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::new(Self::EXE_NAME)),
            man_pages: vec![ManPage::new(1, "rgxg.1")],
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
            a.starts_with("rgxg-") && a.ends_with("x86_64-unknown-linux-musl.tar.gz")
        })?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let binary_data = extractor.extract_by_filename(Self::EXE_NAME)?;
        let man_data = extractor.extract_by_filename("rgxg.1")?;
        let bash_data = extractor.extract_by_filename("rgxg.bash")?;
        let fish_data = extractor.extract_by_filename("rgxg.fish")?;
        let zsh_data = extractor.extract_by_filename("_rgxg")?;
        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data(Self::EXE_NAME, binary_data)),
            man_pages: vec![ManPage::new_with_data(1, "rgxg.1", man_data)],
            completions: vec![
                Completion::new_with_data(Shell::Zsh, Self::EXE_NAME, zsh_data),
                Completion::new_with_data(Shell::Bash, Self::EXE_NAME, bash_data),
                Completion::new_with_data(Shell::Fish, Self::EXE_NAME, fish_data),
            ],
            ..Default::default()
        })
    }
}
