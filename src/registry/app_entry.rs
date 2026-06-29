#[derive(Debug, Clone)]
pub struct ReleasedVersionParseDef {
    pub tag_starts_with: Option<String>,
    pub try_in_body:     bool,
}

#[derive(Debug, Clone)]
pub struct AppEntry {
    pub id:                     String,
    pub category_id:            String,
    pub description:            Option<String>,
    pub url:                    String,
    pub binaries:               Vec<AppBinaryDef>,
    pub assets:                 Vec<AppAssetDef>,
    pub shell_completions:      Vec<ShellCompletionDef>,
    pub man_pages:              Vec<ManPageDef>,
    pub released_version_parse: Option<ReleasedVersionParseDef>,
}

#[derive(Debug, Clone)]
pub struct AppBinaryDef {
    pub id:              u32,
    pub name:            String,
    pub version_cmdline: String,
    pub is_main:         bool,
}

#[derive(Debug, Clone)]
pub struct AppAssetDef {
    pub id:           u32,
    pub asset_type:   AssetType,
    pub starts_with:  Option<String>,
    pub contains:     Option<String>,
    pub not_contains: Option<String>,
    pub ends_with:    Option<String>,
    pub equals:       Option<String>,
}

#[derive(Debug, Clone)]
pub enum AssetType {
    Archive,
    Deb,
    Binary,
}

#[derive(Debug, Clone)]
pub struct ShellCompletionDef {
    pub shell:  ShellKind,
    pub source: CompletionSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ShellKind {
    Bash,
    Zsh,
    Fish,
}

#[derive(Debug, Clone)]
pub enum CompletionSource {
    SelfGenerated { binary_id: u32, command: String },
    Extracted { asset_id: u32, path: String },
}

#[derive(Debug, Clone)]
pub struct ManPageDef {
    pub section: u8,
    pub source:  CompletionSource,
}

impl AppEntry {
    pub fn main_exe_name(&self) -> &str {
        self.binaries
            .iter()
            .find(|b| b.is_main)
            .expect("registry validation ensures exactly one is_main binary")
            .name
            .as_str()
    }

    pub fn has_bundled_man_pages(&self) -> bool {
        self.man_pages
            .iter()
            .any(|mp| matches!(mp.source, CompletionSource::Extracted { .. }))
    }

    pub fn has_bundled_completions(&self) -> bool {
        self.shell_completions
            .iter()
            .any(|sc| matches!(sc.source, CompletionSource::Extracted { .. }))
    }

    pub fn has_declared_musl(&self) -> bool {
        self.assets.iter().any(|a| {
            [&a.starts_with, &a.contains, &a.ends_with, &a.equals]
                .into_iter()
                .filter_map(|f| f.as_deref())
                .any(|s| s.contains("musl"))
        })
    }
}
