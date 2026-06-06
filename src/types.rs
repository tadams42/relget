use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct AppBinary {
    pub name: String,
    pub data: Vec<u8>,
}

impl AppBinary {
    pub fn new(name: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            name: name.into(),
            data,
        }
    }

    pub fn descriptor(name: impl Into<String>) -> Self { Self::new(name, vec![]) }

    pub fn install_path(&self, prefix: &Path) -> PathBuf { prefix.join("bin").join(&self.name) }
}

#[derive(Debug, Clone)]
pub struct ManPage {
    pub section:   u8,
    pub file_name: String,
    pub data:      Vec<u8>,
}

impl ManPage {
    pub fn new(section: u8, file_name: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            section,
            file_name: file_name.into(),
            data,
        }
    }

    pub fn descriptor(section: u8, file_name: impl Into<String>) -> Self {
        Self::new(section, file_name, vec![])
    }

    pub fn install_path(&self, prefix: &Path) -> PathBuf {
        prefix
            .join("share")
            .join("man")
            .join(format!("man{}", self.section))
            .join(&self.file_name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    Zsh,
    Bash,
    Fish,
}

#[derive(Debug, Clone)]
pub struct Completion {
    pub shell:    Shell,
    pub app_name: String,
    pub data:     Vec<u8>,
}

impl Completion {
    pub fn zsh(app_name: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            shell: Shell::Zsh,
            app_name: app_name.into(),
            data,
        }
    }

    pub fn bash(app_name: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            shell: Shell::Bash,
            app_name: app_name.into(),
            data,
        }
    }

    pub fn fish(app_name: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            shell: Shell::Fish,
            app_name: app_name.into(),
            data,
        }
    }

    pub fn zsh_desc(app_name: impl Into<String>) -> Self { Self::zsh(app_name, vec![]) }
    pub fn bash_desc(app_name: impl Into<String>) -> Self { Self::bash(app_name, vec![]) }
    pub fn fish_desc(app_name: impl Into<String>) -> Self { Self::fish(app_name, vec![]) }

    pub fn file_name(&self) -> String {
        match self.shell {
            Shell::Zsh => format!("_{}", self.app_name),
            Shell::Fish => format!("{}.fish", self.app_name),
            Shell::Bash => self.app_name.clone(),
        }
    }

    pub fn install_path(&self, prefix: &Path) -> PathBuf {
        match self.shell {
            Shell::Zsh => {
                prefix
                    .join("share")
                    .join("zsh")
                    .join("site-functions")
                    .join(self.file_name())
            }
            Shell::Fish => {
                prefix
                    .join("share")
                    .join("fish")
                    .join("vendor_completions.d")
                    .join(self.file_name())
            }
            Shell::Bash => {
                prefix
                    .join("share")
                    .join("bash-completion")
                    .join("completions")
                    .join(self.file_name())
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct AppAssets {
    pub binary:      Option<AppBinary>,
    pub other_bins:  Vec<AppBinary>,
    pub man_pages:   Vec<ManPage>,
    pub completions: Vec<Completion>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn prefix() -> PathBuf { PathBuf::from("/usr/local") }

    #[test]
    fn binary_install_path() {
        let b = AppBinary::descriptor("rg");
        assert_eq!(b.install_path(&prefix()), PathBuf::from("/usr/local/bin/rg"));
    }

    #[test]
    fn man_page_install_path_section_1() {
        let m = ManPage::descriptor(1, "rg.1");
        assert_eq!(
            m.install_path(&prefix()),
            PathBuf::from("/usr/local/share/man/man1/rg.1")
        );
    }

    #[test]
    fn completion_file_name_zsh() {
        let c = Completion::zsh_desc("rg");
        assert_eq!(c.file_name(), "_rg");
    }

    #[test]
    fn completion_file_name_bash() {
        let c = Completion::bash_desc("rg");
        assert_eq!(c.file_name(), "rg");
    }

    #[test]
    fn completion_file_name_fish() {
        let c = Completion::fish_desc("rg");
        assert_eq!(c.file_name(), "rg.fish");
    }

    #[test]
    fn completion_install_path_zsh() {
        let c = Completion::zsh_desc("rg");
        assert_eq!(
            c.install_path(&prefix()),
            PathBuf::from("/usr/local/share/zsh/site-functions/_rg")
        );
    }

    #[test]
    fn completion_install_path_bash() {
        let c = Completion::bash_desc("rg");
        assert_eq!(
            c.install_path(&prefix()),
            PathBuf::from("/usr/local/share/bash-completion/completions/rg")
        );
    }

    #[test]
    fn completion_install_path_fish() {
        let c = Completion::fish_desc("rg");
        assert_eq!(
            c.install_path(&prefix()),
            PathBuf::from("/usr/local/share/fish/vendor_completions.d/rg.fish")
        );
    }
}
