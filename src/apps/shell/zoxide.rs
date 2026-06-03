use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppBinary, AppAssets, ManPage};
use crate::version::AppVersion;

pub struct Zoxide {
    client: Arc<GithubClient>,
}

impl Zoxide {
    pub const ID: &'static str = "zoxide";
    const OWNER: &'static str = "ajeetdsouza";
    const REPO: &'static str = "zoxide";
    const EXE_NAME: &'static str = "zoxide";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Zoxide {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary:    Some(AppBinary::descriptor(Self::EXE_NAME)),
            man_pages: vec![
                ManPage::descriptor(1, "zoxide.1"),
                ManPage::descriptor(1, "zoxide-add.1"),
                ManPage::descriptor(1, "zoxide-import.1"),
                ManPage::descriptor(1, "zoxide-init.1"),
                ManPage::descriptor(1, "zoxide-query.1"),
                ManPage::descriptor(1, "zoxide-remove.1"),
            ],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release
            .asset_names()
            .into_iter()
            .find(|a| a.starts_with("zoxide-") && a.ends_with("-x86_64-unknown-linux-musl.tar.gz"))
            .ok_or_else(|| anyhow!("Can't find zoxide musl asset"))?;

        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;

        let exe = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "zoxide")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find zoxide in archive"))?;
        let binary_data = extractor.extract(&exe)?;

        let mut man_pages = Vec::new();
        for member in &members {
            let path = Path::new(member);
            let file_name = path.file_name().and_then(|f| f.to_str()).unwrap_or("");
            if let Some(section_str) = path.extension().and_then(|e| e.to_str()) {
                if let Ok(section) = section_str.parse::<u8>() {
                    man_pages.push(ManPage::new(section, file_name, extractor.extract(member)?));
                }
            }
        }

        Ok(AppAssets {
            binary: Some(AppBinary::new("zoxide", binary_data)),
            man_pages,
            // zoxide init generates completions at shell init time; no static files needed
            ..Default::default()
        })
    }
}
