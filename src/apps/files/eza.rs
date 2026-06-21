use std::path::Path;
use std::sync::Arc;

use anyhow::{Result, anyhow};

use crate::{
    App, AppAssets, AppBinary, AppVersion, ArchiveExtractor, Completion, ManPage, RelgetClient,
    Shell,
};

pub struct Eza {
    client: Arc<dyn RelgetClient>,
}

impl Eza {
    pub const ID: &'static str = "eza";
    const OWNER: &'static str = "eza-community";
    const REPO: &'static str = "eza";
    const EXE_NAME: &'static str = "eza";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
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
            binary: Some(AppBinary::new(Self::EXE_NAME)),
            man_pages: vec![
                ManPage::new(1, "eza.1"),
                ManPage::new(5, "eza_colors.5"),
                ManPage::new(5, "eza_colors-explanation.5"),
            ],
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

        let bin_name = release.find_asset(|a| a == "eza_x86_64-unknown-linux-musl.tar.gz")?;
        let asset = self
            .client
            .download_asset(Self::OWNER, Self::REPO, &bin_name)?;
        let extractor = ArchiveExtractor::new(&bin_name, asset.data);
        let binary_data = extractor.extract_by_filename("eza")?;

        let comp_name =
            release.find_asset(|a| a.starts_with("completions-") && a.ends_with(".tar.gz"))?;
        let comp_asset = self
            .client
            .download_asset(Self::OWNER, Self::REPO, &comp_name)?;
        let comp_extractor = ArchiveExtractor::new(&comp_name, comp_asset.data);
        let comp_members = comp_extractor.members()?;
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
            Completion::new_with_data(
                Shell::Zsh,
                "eza",
                comp_extractor.extract_by_filename("_eza")?,
            ),
            Completion::new_with_data(
                Shell::Fish,
                "eza",
                comp_extractor.extract_by_filename("eza.fish")?,
            ),
            Completion::new_with_data(Shell::Bash, "eza", comp_extractor.extract(&bash_entry)?),
        ];

        let man_name = release.find_asset(|a| a.starts_with("man-") && a.ends_with(".tar.gz"))?;
        let man_asset = self
            .client
            .download_asset(Self::OWNER, Self::REPO, &man_name)?;
        let man_extractor = ArchiveExtractor::new(&man_name, man_asset.data);
        let mut man_pages = Vec::new();
        for member in man_extractor.members()? {
            let path = Path::new(&member);
            let file_name = path.file_name().and_then(|f| f.to_str()).unwrap_or("");
            if let Some(section_str) = path.extension().and_then(|e| e.to_str()) {
                if let Ok(section) = section_str.parse::<u8>() {
                    man_pages.push(ManPage::new_with_data(
                        section,
                        file_name,
                        man_extractor.extract(&member)?,
                    ));
                }
            }
        }

        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data("eza", binary_data)),
            man_pages,
            completions,
            ..Default::default()
        })
    }
}
