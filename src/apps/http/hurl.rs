use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppAssets, AppBinary, Completion, ManPage};
use crate::version::AppVersion;

pub struct Hurl {
    client: Arc<GithubClient>,
}

impl Hurl {
    pub const ID: &'static str = "hurl";
    const OWNER: &'static str = "Orange-OpenSource";
    const REPO: &'static str = "hurl";
    const EXE_NAME: &'static str = "hurl";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
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
            binary:      Some(AppBinary::descriptor(Self::EXE_NAME)),
            other_bins:  vec![AppBinary::descriptor("hurlfmt")],
            man_pages:   vec![
                ManPage::descriptor(1, "hurl.1.gz"),
                ManPage::descriptor(1, "hurlfmt.1.gz"),
            ],
            completions: vec![
                Completion::zsh_desc(Self::EXE_NAME),
                Completion::bash_desc(Self::EXE_NAME),
                Completion::fish_desc(Self::EXE_NAME),
                Completion::zsh_desc("hurlfmt"),
                Completion::bash_desc("hurlfmt"),
                Completion::fish_desc("hurlfmt"),
            ],
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release.find_asset(|a| a.ends_with("-x86_64-unknown-linux-gnu.tar.gz"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let e = ArchiveExtractor::new(&name, asset.data);
        Ok(AppAssets {
            binary:      Some(AppBinary::new("hurl", e.extract_by_filename("hurl")?)),
            other_bins:  vec![AppBinary::new("hurlfmt", e.extract_by_filename("hurlfmt")?)],
            man_pages:   vec![
                ManPage::new(1, "hurl.1.gz", e.extract_by_filename("hurl.1.gz")?),
                ManPage::new(1, "hurlfmt.1.gz", e.extract_by_filename("hurlfmt.1.gz")?),
            ],
            completions: vec![
                Completion::zsh("hurl", e.extract_by_filename("_hurl")?),
                Completion::bash("hurl", e.extract_by_filename("hurl.bash")?),
                Completion::fish("hurl", e.extract_by_filename("hurl.fish")?),
                Completion::zsh("hurlfmt", e.extract_by_filename("_hurlfmt")?),
                Completion::bash("hurlfmt", e.extract_by_filename("hurlfmt.bash")?),
                Completion::fish("hurlfmt", e.extract_by_filename("hurlfmt.fish")?),
            ],
        })
    }
}
