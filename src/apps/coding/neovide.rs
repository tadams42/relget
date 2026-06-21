use std::path::Path;
use std::sync::Arc;

use anyhow::Result;

use crate::{App, AppAssets, AppBinary, AppVersion, ArchiveExtractor, RelgetClient};

pub struct Neovide {
    client: Arc<dyn RelgetClient>,
}

impl Neovide {
    pub const ID: &'static str = "neovide";
    const OWNER: &'static str = "neovide";
    const REPO: &'static str = "neovide";
    const FALLBACK_VERSION: &'static str = "0.15.2";
    const EXE_NAME: &'static str = "neovide";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Neovide {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

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
            .arg(self.cli_version_arg())
            .output();
        match out {
            Err(_) => Ok(Some(AppVersion::parse(Self::FALLBACK_VERSION).unwrap())),
            Ok(o) => {
                if !o.status.success() {
                    log::warn!(
                        "app=neovide fallback={} msg=--version failed (known bug)",
                        Self::FALLBACK_VERSION
                    );
                    return Ok(Some(AppVersion::parse(Self::FALLBACK_VERSION).unwrap()));
                }
                let text = String::from_utf8_lossy(&o.stdout);
                let text2 = String::from_utf8_lossy(&o.stderr);
                let combined = format!("{}{}", text, text2);
                Ok(AppVersion::find_in(&combined))
            }
        }
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::new(Self::EXE_NAME)),
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release.find_asset(|a| {
            a == "neovide-linux-x86_64.tar.gz" || a == "neovide-linux-x86_64.tar"
        })?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data(
                "neovide",
                extractor.extract_by_filename("neovide")?,
            )),
            ..Default::default()
        })
    }
}
