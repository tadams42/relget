use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppBinary, Completion, AppAssets, ManPage};
use crate::version::AppVersion;

pub struct Eza {
    client: Arc<GithubClient>,
}

impl Eza {
    pub const ID: &'static str = "eza";
    pub const CATEGORY: &'static str = "files";
    pub const DESCRIPTION: &'static str =
        "Modern replacement for ls with icons and git integration";
    pub const URL: &'static str = "https://github.com/eza-community/eza";
    const OWNER: &'static str = "eza-community";
    const REPO: &'static str = "eza";
    const EXE_NAME: &'static str = "eza";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Eza {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary:      Some(AppBinary::descriptor("eza")),
            man_pages:   vec![
                ManPage::descriptor(1, "eza.1"),
                ManPage::descriptor(5, "eza_colors.5"),
                ManPage::descriptor(5, "eza_colors-explanation.5"),
            ],
            completions: vec![Completion::zsh_desc("eza"), Completion::bash_desc("eza"), Completion::fish_desc("eza")],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;

        // Binary
        let bin_name = release
            .asset_names()
            .into_iter()
            .find(|a| a == "eza_x86_64-unknown-linux-musl.tar.gz")
            .ok_or_else(|| anyhow!("Can't find eza binary asset"))?;
        let asset = self
            .client
            .download_asset(Self::OWNER, Self::REPO, &bin_name)?;
        let extractor = ArchiveExtractor::new(&bin_name, asset.data);
        let members = extractor.members()?;
        let exe = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "eza")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find eza executable"))?;
        let binary_data = extractor.extract(&exe)?;

        // Completions archive
        let comp_name = release
            .asset_names()
            .into_iter()
            .find(|a| a.starts_with("completions-") && a.ends_with(".tar.gz"))
            .ok_or_else(|| anyhow!("Can't find eza completions asset"))?;
        let comp_asset = self
            .client
            .download_asset(Self::OWNER, Self::REPO, &comp_name)?;
        let comp_extractor = ArchiveExtractor::new(&comp_name, comp_asset.data);
        let comp_members = comp_extractor.members()?;

        let zsh_entry = comp_members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "_eza")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find _eza zsh completion"))?;
        let fish_entry = comp_members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "eza.fish")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find eza.fish completion"))?;
        let bash_entry = comp_members
            .iter()
            .find(|m| {
                let name = Path::new(m)
                    .file_name()
                    .map(|f| f.to_str().unwrap_or(""))
                    .unwrap_or("");
                name == "eza" && m.contains("completions")
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find eza bash completion"))?;

        let completions = vec![
            Completion::zsh("eza", comp_extractor.extract(&zsh_entry)?),
            Completion::fish("eza", comp_extractor.extract(&fish_entry)?),
            Completion::bash("eza", comp_extractor.extract(&bash_entry)?),
        ];

        // Man pages archive
        let man_name = release
            .asset_names()
            .into_iter()
            .find(|a| a.starts_with("man-") && a.ends_with(".tar.gz"))
            .ok_or_else(|| anyhow!("Can't find eza man pages asset"))?;
        let man_asset = self
            .client
            .download_asset(Self::OWNER, Self::REPO, &man_name)?;
        let man_extractor = ArchiveExtractor::new(&man_name, man_asset.data);
        let man_members = man_extractor.members()?;
        let mut man_pages = Vec::new();
        for member in &man_members {
            let path = Path::new(member);
            let file_name = path.file_name().and_then(|f| f.to_str()).unwrap_or("");
            if let Some(section_str) = path.extension().and_then(|e| e.to_str()) {
                if let Ok(section) = section_str.parse::<u8>() {
                    let data = man_extractor.extract(member)?;
                    man_pages.push(ManPage::new(section, file_name, data));
                }
            }
        }

        Ok(AppAssets {
            binary: Some(AppBinary::new("eza", binary_data)),
            man_pages,
            completions,
            ..Default::default()
        })
    }
}
