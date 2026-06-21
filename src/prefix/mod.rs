mod doctor;
mod helpers;
mod installer;
mod syncer;
mod uninstaller;

use std::path::PathBuf;

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Prefix {
    pub path: PathBuf,
}

impl Prefix {
    pub const DEFAULT_PREFIX: &'static str = "/usr/local";

    pub fn new(path: PathBuf) -> Self { Self { path } }

    pub fn install(
        &self, apps: &[String], configured_set: Option<&str>, offline: bool,
    ) -> Result<()> {
        installer::install(&self.path, apps, configured_set, offline)
    }

    pub fn uninstall(&self, apps: &[String], configured_set: Option<&str>) -> Result<()> {
        uninstaller::uninstall(&self.path, apps, configured_set)
    }

    pub fn update(
        &self, apps: &[String], configured_set: Option<&str>, offline: bool,
    ) -> Result<()> {
        installer::update(&self.path, apps, configured_set, offline)
    }

    pub fn sync(&self, apps: &[String], configured_set: Option<&str>, offline: bool) -> Result<()> {
        syncer::sync(&self.path, apps, configured_set, offline)
    }

    pub fn doctor(&self, offline: bool) -> Result<()> { doctor::doctor(&self.path, offline) }
}
