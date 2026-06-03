use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppBinary, Completion, AppAssets, ManPage};
use crate::version::AppVersion;

pub struct Tlrc {
    client: Arc<GithubClient>,
}

impl Tlrc {
    pub const ID: &'static str = "tlrc";
    const OWNER: &'static str = "tldr-pages";
    const REPO: &'static str = "tlrc";
    const EXE_NAME: &'static str = "tldr";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Tlrc {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary:      Some(AppBinary::descriptor(Self::EXE_NAME)),
            man_pages:   vec![ManPage::descriptor(1, "tldr.1")],
            completions: vec![Completion::zsh_desc(Self::EXE_NAME), Completion::bash_desc(Self::EXE_NAME), Completion::fish_desc(Self::EXE_NAME)],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release
            .asset_names()
            .into_iter()
            .find(|a| a.starts_with("tlrc-") && a.ends_with("-x86_64-unknown-linux-gnu.tar.gz"))
            .ok_or_else(|| anyhow!("Can't find tlrc asset"))?;
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

        let exe_entry = find("tldr")?;
        let man_entry = find("tldr.1")?;
        let bash_entry = find("tldr.bash")?;
        let zsh_entry = find("_tldr")?;
        let fish_entry = find("tldr.fish")?;

        Ok(AppAssets {
            binary: Some(AppBinary::new("tldr", extractor.extract(&exe_entry)?)),
            man_pages: vec![ManPage::new(1, "tldr.1", extractor.extract(&man_entry)?)],
            completions: vec![
                Completion::bash("tldr", extractor.extract(&bash_entry)?),
                Completion::zsh("tldr", extractor.extract(&zsh_entry)?),
                Completion::fish("tldr", extractor.extract(&fish_entry)?),
            ],
            ..Default::default()
        })
    }
}
