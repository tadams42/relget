use crate::types::AppAssets;
use crate::version::AppVersion;
use anyhow::Result;
use std::path::Path;

const DEFAULT_VERSION_ARG: &str = "--version";

pub trait App {
    fn exe_name(&self) -> &str;

    fn cli_version_arg(&self) -> &str { DEFAULT_VERSION_ARG }

    fn released_version(&self) -> Result<AppVersion>;

    fn assets(&self) -> AppAssets;

    fn download(&self) -> Result<AppAssets>;

    fn installed_version(&self, prefix: &Path) -> Result<Option<AppVersion>> {
        let bin = prefix.join("bin").join(self.exe_name());
        if !bin.exists() {
            return Ok(None);
        }
        let out = std::process::Command::new(&bin)
            .arg(self.cli_version_arg())
            .output();
        match out {
            Err(_) => Ok(None),
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let stderr = String::from_utf8_lossy(&o.stderr);
                let combined = format!("{}{}", stdout, stderr);
                Ok(AppVersion::find_in(&combined))
            }
        }
    }

    fn needs_install(&self, prefix: &Path) -> Result<bool> {
        let installed = self.installed_version(prefix)?;
        match installed {
            None => Ok(true),
            Some(iv) => Ok(iv != self.released_version()?),
        }
    }

}
