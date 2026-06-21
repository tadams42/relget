use std::sync::Arc;

use anyhow::Result;

use crate::{
    App, AppAssets, AppBinary, AppVersion, ArchiveExtractor, Completion, ManPage, RelgetClient,
    Shell,
};

pub struct Hyperfine {
    client: Arc<dyn RelgetClient>,
}

impl Hyperfine {
    pub const ID: &'static str = "hyperfine";
    const OWNER: &'static str = "sharkdp";
    const REPO: &'static str = "hyperfine";
    const EXE_NAME: &'static str = "hyperfine";

    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Hyperfine {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::new(Self::EXE_NAME)),
            man_pages: vec![ManPage::new(1, "hyperfine.1")],
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
            a.starts_with("hyperfine-") && a.ends_with("-x86_64-unknown-linux-musl.tar.gz")
        })?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data(
                Self::EXE_NAME,
                extractor.extract_by_filename(Self::EXE_NAME)?,
            )),
            man_pages: vec![ManPage::new_with_data(
                1,
                "hyperfine.1",
                extractor.extract_by_filename("hyperfine.1")?,
            )],
            completions: vec![
                Completion::new_with_data(
                    Shell::Bash,
                    Self::EXE_NAME,
                    extractor.extract_by_filename("hyperfine.bash")?,
                ),
                Completion::new_with_data(
                    Shell::Zsh,
                    Self::EXE_NAME,
                    extractor.extract_by_filename("_hyperfine")?,
                ),
                Completion::new_with_data(
                    Shell::Fish,
                    Self::EXE_NAME,
                    extractor.extract_by_filename("hyperfine.fish")?,
                ),
            ],
            ..Default::default()
        })
    }
}
