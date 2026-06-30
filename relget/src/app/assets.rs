use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use registry_core::ShellKind;

fn ensure_parent(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

pub const BIN_MODE: u32 = 0o755;
pub const DOC_MODE: u32 = 0o644;

#[derive(Debug, Clone)]
pub struct Binary {
    name: String,
    data: Vec<u8>,
}

impl Binary {
    pub fn new(name: impl Into<String>) -> Self { Self::new_with_data(name, vec![]) }

    pub fn new_with_data(name: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            name: name.into(),
            data,
        }
    }

    fn install_path(&self, prefix: &Path) -> PathBuf { prefix.join("bin").join(&self.name) }

    pub fn install(&self, prefix: &Path) -> Result<PathBuf> {
        let dest = self.install_path(prefix);
        ensure_parent(&dest)?;
        // Write to a temp file then atomically rename into place, matching the standard approach
        // used by dpkg, rpm, and Homebrew. Direct in-place write (O_TRUNC) returns ETXTBSY if any
        // process holds the binary exec-mapped — even a short-lived one. For example, `boring`
        // spawns 5 child processes on startup; if they are still alive when we write, the install
        // fails. rename() replaces the directory entry without touching the existing inode, so any
        // running process keeps its mapping on the old inode while the new binary is already in
        // place.
        let tmp = dest.with_extension("relget-tmp");
        fs::write(&tmp, &self.data).with_context(|| format!("Writing binary to {:?}", dest))?;
        fs::set_permissions(&tmp, fs::Permissions::from_mode(BIN_MODE))?;
        fs::rename(&tmp, &dest).with_context(|| format!("Installing binary to {:?}", dest))?;
        Ok(dest)
    }

    pub fn uninstall(&self, prefix: &Path) -> Option<PathBuf> {
        let path = self.install_path(prefix);
        if fs::remove_file(&path).is_ok() {
            Some(path)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct ManPage {
    section:   u8,
    file_name: String,
    data:      Vec<u8>,
}

impl ManPage {
    pub fn new(section: u8, file_name: impl Into<String>) -> Self {
        Self::new_with_data(section, file_name, vec![])
    }

    pub fn new_with_data(section: u8, file_name: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            section,
            file_name: file_name.into(),
            data,
        }
    }

    fn install_path(&self, prefix: &Path) -> PathBuf {
        prefix
            .join("share")
            .join("man")
            .join(format!("man{}", self.section))
            .join(&self.file_name)
    }

    pub fn install(&self, prefix: &Path) -> Result<PathBuf> {
        let dest = self.install_path(prefix);
        ensure_parent(&dest)?;
        fs::write(&dest, &self.data).with_context(|| format!("Writing man page to {:?}", dest))?;
        fs::set_permissions(&dest, fs::Permissions::from_mode(DOC_MODE))?;
        Ok(dest)
    }

    pub fn uninstall(&self, prefix: &Path) -> Option<PathBuf> {
        let path = self.install_path(prefix);
        if fs::remove_file(&path).is_ok() {
            Some(path)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct ShellCompletion {
    shell:    ShellKind,
    app_name: String,
    data:     Vec<u8>,
}

impl ShellCompletion {
    pub fn new(shell: ShellKind, app_name: impl Into<String>) -> Self {
        Self {
            shell,
            app_name: app_name.into(),
            data: vec![],
        }
    }

    pub fn new_with_data(shell: ShellKind, app_name: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            shell,
            app_name: app_name.into(),
            data,
        }
    }

    fn file_name(&self) -> String {
        match self.shell {
            ShellKind::Zsh => format!("_{}", self.app_name),
            ShellKind::Fish => format!("{}.fish", self.app_name),
            ShellKind::Bash => self.app_name.clone(),
        }
    }

    fn install_path(&self, prefix: &Path) -> PathBuf {
        match self.shell {
            ShellKind::Zsh => {
                prefix
                    .join("share")
                    .join("zsh")
                    .join("site-functions")
                    .join(self.file_name())
            }
            ShellKind::Fish => {
                prefix
                    .join("share")
                    .join("fish")
                    .join("vendor_completions.d")
                    .join(self.file_name())
            }
            ShellKind::Bash => {
                prefix
                    .join("share")
                    .join("bash-completion")
                    .join("completions")
                    .join(self.file_name())
            }
        }
    }

    pub fn install(&self, prefix: &Path) -> Result<PathBuf> {
        let dest = self.install_path(prefix);
        ensure_parent(&dest)?;
        fs::write(&dest, &self.data)
            .with_context(|| format!("Writing completion to {:?}", dest))?;
        fs::set_permissions(&dest, fs::Permissions::from_mode(DOC_MODE))?;
        Ok(dest)
    }

    pub fn uninstall(&self, prefix: &Path) -> Option<PathBuf> {
        let path = self.install_path(prefix);
        if fs::remove_file(&path).is_ok() {
            Some(path)
        } else {
            None
        }
    }
}

#[derive(Debug, Default)]
pub struct Assets {
    pub binary:      Option<Binary>,
    pub other_bins:  Vec<Binary>,
    pub man_pages:   Vec<ManPage>,
    pub completions: Vec<ShellCompletion>,
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn prefix() -> PathBuf { PathBuf::from("/usr/local") }

    #[test]
    fn binary_install_path() {
        let b = Binary::new("rg");
        assert_eq!(b.install_path(&prefix()), PathBuf::from("/usr/local/bin/rg"));
    }

    #[test]
    fn man_page_install_path_section_1() {
        let m = ManPage::new(1, "rg.1");
        assert_eq!(
            m.install_path(&prefix()),
            PathBuf::from("/usr/local/share/man/man1/rg.1")
        );
    }

    #[test]
    fn completion_file_name_zsh() {
        let c = ShellCompletion::new(ShellKind::Zsh, "rg");
        assert_eq!(c.file_name(), "_rg");
    }

    #[test]
    fn completion_file_name_bash() {
        let c = ShellCompletion::new(ShellKind::Bash, "rg");
        assert_eq!(c.file_name(), "rg");
    }

    #[test]
    fn completion_file_name_fish() {
        let c = ShellCompletion::new(ShellKind::Fish, "rg");
        assert_eq!(c.file_name(), "rg.fish");
    }

    #[test]
    fn completion_install_path_zsh() {
        let c = ShellCompletion::new(ShellKind::Zsh, "rg");
        assert_eq!(
            c.install_path(&prefix()),
            PathBuf::from("/usr/local/share/zsh/site-functions/_rg")
        );
    }

    #[test]
    fn completion_install_path_bash() {
        let c = ShellCompletion::new(ShellKind::Bash, "rg");
        assert_eq!(
            c.install_path(&prefix()),
            PathBuf::from("/usr/local/share/bash-completion/completions/rg")
        );
    }

    #[test]
    fn completion_install_path_fish() {
        let c = ShellCompletion::new(ShellKind::Fish, "rg");
        assert_eq!(
            c.install_path(&prefix()),
            PathBuf::from("/usr/local/share/fish/vendor_completions.d/rg.fish")
        );
    }
}
