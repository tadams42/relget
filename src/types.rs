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
pub struct DownloadedAssets {
    pub binary:      Option<AppBinary>,
    pub other_bins:  Vec<AppBinary>,
    pub man_pages:   Vec<ManPage>,
    pub completions: Vec<Completion>,
}
