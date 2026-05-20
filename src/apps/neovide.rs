use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::github::GithubClient;
use crate::types::{AppBinary, DownloadedAssets};
use crate::version::AppVersion;

pub struct Neovide {
    client: Arc<GithubClient>,
}

impl Neovide {
    const OWNER: &'static str = "neovide";
    const REPO: &'static str = "neovide";
    const FALLBACK_VERSION: &'static str = "0.15.2";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Neovide {
    fn exe_name(&self) -> &str { "neovide" }
    fn url(&self) -> &str { "https://github.com/neovide/neovide" }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn installed_version(&self, prefix: &Path) -> Result<Option<AppVersion>> {
        let bin = prefix.join("bin").join(self.exe_name());
        if !bin.exists() {
            return Ok(None);
        }
        let out = std::process::Command::new(&bin)
            .arg(self.installed_version_flag())
            .output();
        match out {
            Err(_) => Ok(Some(AppVersion::parse(Self::FALLBACK_VERSION).unwrap())),
            Ok(o) => {
                if !o.status.success() {
                    log::warn!(
                        "neovide --version failed (known bug), assuming {}",
                        Self::FALLBACK_VERSION
                    );
                    return Ok(Some(AppVersion::parse(Self::FALLBACK_VERSION).unwrap()));
                }
                let text = String::from_utf8_lossy(&o.stdout);
                let text2 = String::from_utf8_lossy(&o.stderr);
                let combined = format!("{}{}", text, text2);
                Ok(self.parse_installed_version(&combined))
            }
        }
    }

    fn download(&self) -> Result<DownloadedAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release
            .asset_names()
            .into_iter()
            .find(|a| a == "neovide-linux-x86_64.tar.gz" || a == "neovide-linux-x86_64.tar")
            .ok_or_else(|| anyhow!("Can't find neovide Linux asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;
        let exe = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "neovide")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find neovide in archive"))?;
        Ok(DownloadedAssets {
            binary: Some(AppBinary::new("neovide", extractor.extract(&exe)?)),
            ..Default::default()
        })
    }
}
