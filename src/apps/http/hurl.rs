use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppBinary, Completion, AppAssets, ManPage};
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
            binary:      Some(AppBinary::descriptor("hurl")),
            other_bins:  vec![AppBinary::descriptor("hurlfmt")],
            man_pages:   vec![
                ManPage::descriptor(1, "hurl.1.gz"),
                ManPage::descriptor(1, "hurlfmt.1.gz"),
            ],
            completions: vec![
                Completion::zsh_desc("hurl"),
                Completion::bash_desc("hurl"),
                Completion::fish_desc("hurl"),
                Completion::zsh_desc("hurlfmt"),
                Completion::bash_desc("hurlfmt"),
                Completion::fish_desc("hurlfmt"),
            ],
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release
            .asset_names()
            .into_iter()
            .find(|a| a.ends_with("-x86_64-unknown-linux-gnu.tar.gz"))
            .ok_or_else(|| anyhow!("Can't find hurl Linux x86_64 asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;

        let find = |file: &str| -> Result<String> {
            members
                .iter()
                .find(|m| Path::new(m).file_name().map(|f| f == file).unwrap_or(false))
                .cloned()
                .ok_or_else(|| anyhow!("Can't find {} in archive", file))
        };

        let hurl_data = extractor.extract(&find("hurl")?)?;
        let hurlfmt_data = extractor.extract(&find("hurlfmt")?)?;

        let man_pages = vec![
            ManPage::new(1, "hurl.1.gz", extractor.extract(&find("hurl.1.gz")?)?),
            ManPage::new(1, "hurlfmt.1.gz", extractor.extract(&find("hurlfmt.1.gz")?)?),
        ];

        let completions = vec![
            Completion::zsh("hurl", extractor.extract(&find("_hurl")?)?),
            Completion::bash("hurl", extractor.extract(&find("hurl.bash")?)?),
            Completion::fish("hurl", extractor.extract(&find("hurl.fish")?)?),
            Completion::zsh("hurlfmt", extractor.extract(&find("_hurlfmt")?)?),
            Completion::bash("hurlfmt", extractor.extract(&find("hurlfmt.bash")?)?),
            Completion::fish("hurlfmt", extractor.extract(&find("hurlfmt.fish")?)?),
        ];

        Ok(AppAssets {
            binary: Some(AppBinary::new("hurl", hurl_data)),
            other_bins: vec![AppBinary::new("hurlfmt", hurlfmt_data)],
            man_pages,
            completions,
        })
    }
}
