use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::apps::app_assets::{AppAssets, AppBinary, Completion, ManPage, Shell};
use crate::archive::ArchiveExtractor;
use crate::clients::RelgetClient;
use crate::version::AppVersion;

pub struct Hurl {
    client: Arc<dyn RelgetClient>,
}

impl Hurl {
    pub const ID: &'static str = "hurl";
    const OWNER: &'static str = "Orange-OpenSource";
    const REPO: &'static str = "hurl";
    const EXE_NAME: &'static str = "hurl";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Hurl {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary:      Some(AppBinary::new(Self::EXE_NAME)),
            other_bins:  vec![AppBinary::new("hurlfmt")],
            man_pages:   vec![
                ManPage::new(1, "hurl.1.gz"),
                ManPage::new(1, "hurlfmt.1.gz"),
            ],
            completions: vec![
                Completion::new(Shell::Zsh, Self::EXE_NAME),
                Completion::new(Shell::Bash, Self::EXE_NAME),
                Completion::new(Shell::Fish, Self::EXE_NAME),
                Completion::new(Shell::Zsh, "hurlfmt"),
                Completion::new(Shell::Bash, "hurlfmt"),
                Completion::new(Shell::Fish, "hurlfmt"),
            ],
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release.find_asset(|a| a.ends_with("-x86_64-unknown-linux-gnu.tar.gz"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let e = ArchiveExtractor::new(&name, asset.data);
        Ok(AppAssets {
            binary:      Some(AppBinary::new_with_data("hurl", e.extract_by_filename("hurl")?)),
            other_bins:  vec![AppBinary::new_with_data(
                "hurlfmt",
                e.extract_by_filename("hurlfmt")?,
            )],
            man_pages:   vec![
                ManPage::new_with_data(1, "hurl.1.gz", e.extract_by_filename("hurl.1.gz")?),
                ManPage::new_with_data(1, "hurlfmt.1.gz", e.extract_by_filename("hurlfmt.1.gz")?),
            ],
            completions: vec![
                Completion::new_with_data(Shell::Zsh, "hurl", e.extract_by_filename("_hurl")?),
                Completion::new_with_data(Shell::Bash, "hurl", e.extract_by_filename("hurl.bash")?),
                Completion::new_with_data(Shell::Fish, "hurl", e.extract_by_filename("hurl.fish")?),
                Completion::new_with_data(
                    Shell::Zsh,
                    "hurlfmt",
                    e.extract_by_filename("_hurlfmt")?,
                ),
                Completion::new_with_data(
                    Shell::Bash,
                    "hurlfmt",
                    e.extract_by_filename("hurlfmt.bash")?,
                ),
                Completion::new_with_data(
                    Shell::Fish,
                    "hurlfmt",
                    e.extract_by_filename("hurlfmt.fish")?,
                ),
            ],
        })
    }
}
