use std::path::Path;
use std::sync::Arc;

use anyhow::{Result, anyhow};

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::RelgetClient;
use crate::types::{AppAssets, AppBinary, Completion, ManPage};
use crate::version::AppVersion;

pub struct Bottom {
    client: Arc<dyn RelgetClient>,
}

impl Bottom {
    pub const ID: &'static str = "bottom";
    const OWNER: &'static str = "ClementTsang";
    const REPO: &'static str = "bottom";
    const EXE_NAME: &'static str = "btm";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Bottom {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::descriptor(Self::EXE_NAME)),
            man_pages: vec![ManPage::descriptor(1, "btm.1")],
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

        // Binary: x86_64 musl static build
        let bin_name = release
            .asset_names()
            .into_iter()
            .find(|a| a.contains("x86_64-unknown-linux-musl") && a.ends_with(".tar.gz"))
            .ok_or_else(|| anyhow!("Can't find bottom musl binary asset"))?;
        let asset = self
            .client
            .download_asset(Self::OWNER, Self::REPO, &bin_name)?;
        let extractor = ArchiveExtractor::new(&bin_name, asset.data);
        let binary_data = extractor.extract_by_filename(Self::EXE_NAME)?;

        // Man page from manpage.tar.gz — contains btm.1.gz
        let man_asset_name = release
            .asset_names()
            .into_iter()
            .find(|a| a == "manpage.tar.gz")
            .ok_or_else(|| anyhow!("Can't find manpage.tar.gz asset"))?;
        let man_asset = self
            .client
            .download_asset(Self::OWNER, Self::REPO, &man_asset_name)?;
        let man_extractor = ArchiveExtractor::new(&man_asset_name, man_asset.data);
        let man_members = man_extractor.members()?;
        let man_gz_entry = man_members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "btm.1.gz")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find btm.1.gz in manpage.tar.gz"))?;
        let compressed = man_extractor.extract(&man_gz_entry)?;
        let decompressed = ArchiveExtractor::new("man.gz", compressed).extract("man")?;
        let man_pages = vec![ManPage::new(1, "btm.1", decompressed)];

        // Completions from completion.tar.gz — contains btm.bash, _btm, btm.fish
        let comp_asset_name = release
            .asset_names()
            .into_iter()
            .find(|a| a == "completion.tar.gz")
            .ok_or_else(|| anyhow!("Can't find completion.tar.gz asset"))?;
        let comp_asset = self
            .client
            .download_asset(Self::OWNER, Self::REPO, &comp_asset_name)?;
        let comp_extractor = ArchiveExtractor::new(&comp_asset_name, comp_asset.data);
        let comp_members = comp_extractor.members()?;

        let mut completions = Vec::new();
        if let Some(m) = comp_members.iter().find(|m| {
            Path::new(m)
                .file_name()
                .map(|f| f == "btm.bash")
                .unwrap_or(false)
        }) {
            completions.push(Completion::bash(Self::EXE_NAME, comp_extractor.extract(m)?));
        }
        if let Some(m) = comp_members.iter().find(|m| {
            Path::new(m)
                .file_name()
                .map(|f| f == "_btm")
                .unwrap_or(false)
        }) {
            completions.push(Completion::zsh(Self::EXE_NAME, comp_extractor.extract(m)?));
        }
        if let Some(m) = comp_members.iter().find(|m| {
            Path::new(m)
                .file_name()
                .map(|f| f == "btm.fish")
                .unwrap_or(false)
        }) {
            completions.push(Completion::fish(Self::EXE_NAME, comp_extractor.extract(m)?));
        }

        Ok(AppAssets {
            binary: Some(AppBinary::new(Self::EXE_NAME, binary_data)),
            man_pages,
            completions,
            ..Default::default()
        })
    }
}
