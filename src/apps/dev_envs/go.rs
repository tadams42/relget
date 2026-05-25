use crate::apps::App;
use crate::types::DownloadedAssets;
use crate::version::AppVersion;
use anyhow::{Context, Result, anyhow};
use std::path::{Path, PathBuf};

const DOWNLOAD_VERSION: &str = "1.26.0";
const DOWNLOAD_URL: &str = "https://go.dev/dl/go1.26.0.linux-amd64.tar.gz";
const DOWNLOAD_TARBALL: &str = "go1.26.0.linux-amd64.tar.gz";

pub struct Go {
    cache_path: PathBuf,
}

impl Default for Go {
    fn default() -> Self { Self::new() }
}

impl Go {
    pub const DESCRIPTION: &'static str = "Go programming language toolchain";
    pub const URL: &'static str = "https://go.dev/";
    const EXE_NAME: &'static str = "go";
    const VERSION_ARG: &'static str = "version";
    pub fn new() -> Self {
        let cache_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".cache")
            .join("relget");
        Self {
            cache_path: cache_dir.join(DOWNLOAD_TARBALL),
        }
    }
}

impl App for Go {
    fn exe_name(&self) -> &str { Self::EXE_NAME }
    fn cli_version_arg(&self) -> &str { Self::VERSION_ARG }

    fn released_version(&self) -> Result<AppVersion> {
        AppVersion::parse(DOWNLOAD_VERSION)
            .ok_or_else(|| anyhow!("Can't parse Go version {}", DOWNLOAD_VERSION))
    }

    fn download(&self) -> Result<DownloadedAssets> {
        // Go installs a directory; download returns nothing usable for standard install
        if !self.cache_path.exists() {
            log::info!("app=go msg=Downloading Go v{}", DOWNLOAD_VERSION);
            if let Some(parent) = self.cache_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let resp = ureq::get(DOWNLOAD_URL)
                .header("User-Agent", "relget")
                .call()
                .context("Downloading Go tarball")?;
            let buf = resp
                .into_body()
                .read_to_vec()
                .context("Downloading Go tarball body")?;
            std::fs::write(&self.cache_path, &buf)?;
            log::info!("app=go msg=Downloaded Go v{}", DOWNLOAD_VERSION);
        } else {
            log::info!("app=go msg=Using cached Go v{}", DOWNLOAD_VERSION);
        }
        Ok(DownloadedAssets::default())
    }

    fn install(&self, prefix: &Path) -> Result<Vec<PathBuf>> {
        if !self.needs_install(prefix)? {
            log::info!("app=go msg=Already at latest version");
            return Ok(vec![]);
        }

        self.download()?;

        let installed_dir = prefix.join("go");
        let installed_symlink = prefix.join("bin").join("go");

        if installed_dir.exists() {
            std::fs::remove_dir_all(&installed_dir)?;
        }
        if installed_symlink.exists() {
            std::fs::remove_file(&installed_symlink)?;
        }

        // Extract via system tar
        std::process::Command::new("/usr/bin/tar")
            .args([
                "--directory",
                prefix.to_str().unwrap(),
                "--extract",
                "--file",
                self.cache_path.to_str().unwrap(),
            ])
            .status()
            .context("Running tar to extract Go")?;

        std::fs::create_dir_all(prefix.join("bin"))?;
        std::os::unix::fs::symlink(installed_dir.join("bin").join("go"), &installed_symlink)?;

        log::info!("app=go msg=Installed Go v{}", DOWNLOAD_VERSION);
        Ok(vec![installed_dir, installed_symlink])
    }
}
