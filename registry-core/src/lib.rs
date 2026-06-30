use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleasedVersionParseDef {
    pub tag_starts_with: Option<String>,
    pub try_in_body:     bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppBinaryDef {
    pub id:              u32,
    pub name:            String,
    pub version_cmdline: String,
    pub is_main:         bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppAssetDef {
    pub id:           u32,
    pub asset_type:   AssetType,
    pub starts_with:  Option<String>,
    pub contains:     Option<String>,
    pub not_contains: Option<String>,
    pub ends_with:    Option<String>,
    pub equals:       Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssetType {
    Archive,
    Deb,
    Binary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellCompletionDef {
    pub shell:  ShellKind,
    pub source: CompletionSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShellKind {
    Bash,
    Zsh,
    Fish,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompletionSource {
    SelfGenerated { binary_id: u32, command: String },
    Extracted { asset_id: u32, path: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManPageDef {
    pub section: u8,
    pub source:  CompletionSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryEntry {
    pub id:          String,
    pub title:       String,
    pub description: Option<String>,
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

/// Returns a list of semantic rule violations; empty means valid.
pub fn validate(apps: &[AppEntry], categories: &[CategoryEntry]) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();

    let category_ids: HashSet<&str> = categories.iter().map(|c| c.id.as_str()).collect();

    let mut global_binary_names: HashMap<String, String> = HashMap::new();
    let mut global_sc_gen: HashMap<(String, ShellKind), String> = HashMap::new();
    let mut global_sc_ext: HashMap<String, String> = HashMap::new();
    let mut global_mp_gen: HashMap<(String, String), String> = HashMap::new();
    let mut global_mp_ext: HashMap<String, String> = HashMap::new();

    for app in apps {
        let app_id = &app.id;

        if !category_ids.contains(app.category_id.as_str()) {
            errors.push(format!("{app_id}: unknown category_id '{}'", app.category_id));
        }

        let binary_ids: HashSet<u32> = app.binaries.iter().map(|b| b.id).collect();
        let asset_ids: HashSet<u32> = app.assets.iter().map(|a| a.id).collect();

        if binary_ids.len() != app.binaries.len() {
            errors.push(format!("{app_id}: duplicate binary ids"));
        }

        {
            let mut seen: HashSet<&str> = HashSet::new();
            for b in &app.binaries {
                if !seen.insert(b.name.as_str()) {
                    errors.push(format!("{app_id}: duplicate binary name '{}'", b.name));
                }
            }
        }

        if asset_ids.len() != app.assets.len() {
            errors.push(format!("{app_id}: duplicate asset ids"));
        }

        let main_count = app.binaries.iter().filter(|b| b.is_main).count();
        if main_count != 1 {
            errors.push(format!(
                "{app_id}: expected exactly 1 binary with is_main=true, found {main_count}"
            ));
        }

        for b in &app.binaries {
            match global_binary_names.entry(b.name.clone()) {
                std::collections::hash_map::Entry::Vacant(e) => {
                    e.insert(app_id.clone());
                }
                std::collections::hash_map::Entry::Occupied(e) => {
                    let other = e.get();
                    if other != app_id {
                        errors.push(format!(
                            "{app_id}: binary name '{}' conflicts with app '{other}'",
                            b.name
                        ));
                    }
                }
            }
        }

        let mut sc_gen_seen: HashSet<(u32, ShellKind)> = HashSet::new();
        let mut sc_ext_seen: HashSet<String> = HashSet::new();

        for sc in &app.shell_completions {
            match &sc.source {
                CompletionSource::SelfGenerated { binary_id, .. } => {
                    if !binary_ids.contains(binary_id) {
                        errors.push(format!(
                            "{app_id}: shell_completion references unknown binary_id {binary_id}"
                        ));
                    }
                    if !sc_gen_seen.insert((*binary_id, sc.shell.clone())) {
                        errors.push(format!(
                            "{app_id}: duplicate self_generated completion for \
                             binary_id={binary_id} shell={:?}",
                            sc.shell
                        ));
                    }
                    let bin_name = app
                        .binaries
                        .iter()
                        .find(|b| b.id == *binary_id)
                        .map(|b| b.name.clone());
                    if let Some(name) = bin_name {
                        let gkey = (name.clone(), sc.shell.clone());
                        match global_sc_gen.entry(gkey) {
                            std::collections::hash_map::Entry::Vacant(e) => {
                                e.insert(app_id.clone());
                            }
                            std::collections::hash_map::Entry::Occupied(e) => {
                                let other = e.get();
                                if other != app_id {
                                    errors.push(format!(
                                        "{app_id}: self_generated completion for '{name}' \
                                         {:?} conflicts with app '{other}'",
                                        sc.shell
                                    ));
                                }
                            }
                        }
                    }
                }
                CompletionSource::Extracted { asset_id, path } => {
                    if !asset_ids.contains(asset_id) {
                        errors.push(format!(
                            "{app_id}: shell_completion references unknown asset_id {asset_id}"
                        ));
                    }
                    if !sc_ext_seen.insert(path.clone()) {
                        errors.push(format!(
                            "{app_id}: duplicate extracted completion path '{path}'"
                        ));
                    }
                    match global_sc_ext.entry(path.clone()) {
                        std::collections::hash_map::Entry::Vacant(e) => {
                            e.insert(app_id.clone());
                        }
                        std::collections::hash_map::Entry::Occupied(e) => {
                            let other = e.get();
                            if other != app_id {
                                errors.push(format!(
                                    "{app_id}: extracted completion path '{path}' conflicts \
                                     with app '{other}'"
                                ));
                            }
                        }
                    }
                }
            }
        }

        let mut mp_gen_seen: HashSet<(u32, String)> = HashSet::new();
        let mut mp_ext_seen: HashSet<String> = HashSet::new();

        for mp in &app.man_pages {
            match &mp.source {
                CompletionSource::SelfGenerated { binary_id, command } => {
                    if !binary_ids.contains(binary_id) {
                        errors.push(format!(
                            "{app_id}: man_page references unknown binary_id {binary_id}"
                        ));
                    }
                    if !mp_gen_seen.insert((*binary_id, command.clone())) {
                        errors.push(format!(
                            "{app_id}: duplicate self_generated man page for \
                             binary_id={binary_id} command='{command}'"
                        ));
                    }
                    let bin_name = app
                        .binaries
                        .iter()
                        .find(|b| b.id == *binary_id)
                        .map(|b| b.name.clone());
                    if let Some(name) = bin_name {
                        let gkey = (name.clone(), command.clone());
                        match global_mp_gen.entry(gkey) {
                            std::collections::hash_map::Entry::Vacant(e) => {
                                e.insert(app_id.clone());
                            }
                            std::collections::hash_map::Entry::Occupied(e) => {
                                let other = e.get();
                                if other != app_id {
                                    errors.push(format!(
                                        "{app_id}: self_generated man page for '{name}' \
                                         command='{command}' conflicts with app '{other}'"
                                    ));
                                }
                            }
                        }
                    }
                }
                CompletionSource::Extracted { asset_id, path } => {
                    if !asset_ids.contains(asset_id) {
                        errors.push(format!(
                            "{app_id}: man_page references unknown asset_id {asset_id}"
                        ));
                    }
                    if !mp_ext_seen.insert(path.clone()) {
                        errors
                            .push(format!("{app_id}: duplicate extracted man page path '{path}'"));
                    }
                    match global_mp_ext.entry(path.clone()) {
                        std::collections::hash_map::Entry::Vacant(e) => {
                            e.insert(app_id.clone());
                        }
                        std::collections::hash_map::Entry::Occupied(e) => {
                            let other = e.get();
                            if other != app_id {
                                errors.push(format!(
                                    "{app_id}: extracted man page path '{path}' conflicts \
                                     with app '{other}'"
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    errors
}
