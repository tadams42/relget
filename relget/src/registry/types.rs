// All registry type definitions (`AppEntry`, `CategoryEntry`, etc.), `impl AppEntry` helpers, and
// the semantic `validate()` function live in `relget/src/registry/types.rs`. That file is shared
// between the runtime crate (as a normal module) and `relget/build.rs` (via `#[path]` inclusion),
// so the build script and the binary agree on one set of types and one validation implementation —
// it must stay self-contained (std + serde only).
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
    pub conflicts:              Vec<String>,
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
/// Not called by the runtime crate itself — only by `build.rs` and unit tests.
#[cfg_attr(not(test), allow(dead_code))]
pub fn validate(apps: &[AppEntry], categories: &[CategoryEntry]) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();

    let category_ids: HashSet<&str> = categories.iter().map(|c| c.id.as_str()).collect();

    let app_ids: HashSet<&str> = apps.iter().map(|a| a.id.as_str()).collect();
    let conflicts_of: HashMap<&str, HashSet<&str>> = apps
        .iter()
        .map(|a| (a.id.as_str(), a.conflicts.iter().map(String::as_str).collect()))
        .collect();
    let binary_names_of: HashMap<&str, HashSet<&str>> = apps
        .iter()
        .map(|a| (a.id.as_str(), a.binaries.iter().map(|b| b.name.as_str()).collect()))
        .collect();
    let mutual = |a: &str, b: &str| {
        conflicts_of.get(a).is_some_and(|s| s.contains(b))
            && conflicts_of.get(b).is_some_and(|s| s.contains(a))
    };

    let mut global_binary_names: HashMap<String, Vec<String>> = HashMap::new();
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

        for c in &app.conflicts {
            if c == app_id {
                errors.push(format!("{app_id}: conflicts lists itself"));
                continue;
            }
            if !app_ids.contains(c.as_str()) {
                errors.push(format!("{app_id}: conflicts references unknown app '{c}'"));
                continue;
            }
            if !conflicts_of[c.as_str()].contains(app_id.as_str()) {
                errors.push(format!(
                    "{app_id}: conflict with '{c}' is not mutual ('{c}' must also list \
                     '{app_id}' in conflicts)"
                ));
            } else if app_id.as_str() < c.as_str() // report each pair once
                && binary_names_of[app_id.as_str()] == binary_names_of[c.as_str()]
            {
                errors.push(format!(
                    "{app_id}: conflicts with '{c}' but their binary name sets are identical; \
                     conflicting apps must have distinguishable binaries"
                ));
            }
        }

        for b in &app.binaries {
            let holders = global_binary_names.entry(b.name.clone()).or_default();
            for other in holders.iter() {
                if other != app_id && !mutual(app_id, other) {
                    errors.push(format!(
                        "{app_id}: binary name '{}' conflicts with app '{other}' (apps sharing \
                         a binary name must list each other in 'conflicts')",
                        b.name
                    ));
                }
            }
            holders.push(app_id.clone());
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

// ===== Tests =====

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Helpers =====

    fn test_categories() -> Vec<CategoryEntry> {
        vec![CategoryEntry {
            id:          "test".into(),
            title:       "Test".into(),
            description: None,
        }]
    }

    fn make_app(id: &str) -> AppEntry {
        AppEntry {
            id:                     id.into(),
            category_id:            "test".into(),
            description:            None,
            url:                    "https://example.com".into(),
            binaries:               vec![AppBinaryDef {
                id:              1,
                name:            id.into(),
                version_cmdline: "--version".into(),
                is_main:         true,
            }],
            assets:                 vec![AppAssetDef {
                id:           1,
                asset_type:   AssetType::Archive,
                starts_with:  None,
                contains:     None,
                not_contains: None,
                ends_with:    None,
                equals:       Some("foo.tar.gz".into()),
            }],
            shell_completions:      vec![],
            man_pages:              vec![],
            conflicts:              vec![],
            released_version_parse: None,
        }
    }

    fn no_errors(errors: &[String]) {
        assert!(errors.is_empty(), "expected no errors but got: {errors:#?}");
    }

    fn has_error(errors: &[String], fragment: &str) {
        assert!(
            errors.iter().any(|e| e.contains(fragment)),
            "expected error containing {fragment:?} but got: {errors:#?}"
        );
    }

    // ===== Semantic rule tests =====

    // Rule 1: binary name uniqueness within app

    #[test]
    fn rule1_binary_names_unique_within_app_ok() {
        let mut app = make_app("foo");
        app.binaries.push(AppBinaryDef {
            id:              2,
            name:            "foox".into(),
            version_cmdline: "--version".into(),
            is_main:         false,
        });
        no_errors(&validate(&[app], &test_categories()));
    }

    #[test]
    fn rule1_binary_names_duplicate_within_app() {
        let mut app = make_app("foo");
        app.binaries.push(AppBinaryDef {
            id:              2,
            name:            "foo".into(), // same name as binary id=1
            version_cmdline: "--version".into(),
            is_main:         false,
        });
        has_error(&validate(&[app], &test_categories()), "duplicate binary name");
    }

    // Rule 2: binary name uniqueness globally

    #[test]
    fn rule2_binary_names_unique_globally_ok() {
        let app_a = make_app("aaa");
        let app_b = make_app("bbb");
        no_errors(&validate(&[app_a, app_b], &test_categories()));
    }

    #[test]
    fn rule2_binary_names_conflict_globally() {
        let app_a = make_app("shared");
        let mut app_b = make_app("other");
        // binary in app_b named "shared" conflicts with app_a's binary
        app_b.binaries[0].name = "shared".into();
        has_error(&validate(&[app_a, app_b], &test_categories()), "conflicts with app");
    }

    // Rule 7: conflicts (apps sharing binary names must mutually declare each other)

    /// Two apps sharing a main binary name, distinguishable by app_b's extra binary.
    fn make_conflicting_pair() -> (AppEntry, AppEntry) {
        let mut app_a = make_app("app-a");
        app_a.binaries[0].name = "shared".into();
        app_a.conflicts = vec!["app-b".into()];
        let mut app_b = make_app("app-b");
        app_b.binaries[0].name = "shared".into();
        app_b.binaries.push(AppBinaryDef {
            id:              2,
            name:            "shared-extra".into(),
            version_cmdline: "--version".into(),
            is_main:         false,
        });
        app_b.conflicts = vec!["app-a".into()];
        (app_a, app_b)
    }

    #[test]
    fn rule7_mutual_shared_main_binary_ok() {
        let (app_a, app_b) = make_conflicting_pair();
        no_errors(&validate(&[app_a, app_b], &test_categories()));
    }

    #[test]
    fn rule7_mutual_shared_secondary_binary_ok() {
        // the shared name is on a non-main binary of app_b
        let (app_a, mut app_b) = make_conflicting_pair();
        app_b.binaries[0].name = "app-b".into();
        app_b.binaries[1].name = "shared".into();
        no_errors(&validate(&[app_a, app_b], &test_categories()));
    }

    #[test]
    fn rule7_one_sided_listing_errors() {
        let (app_a, mut app_b) = make_conflicting_pair();
        app_b.conflicts = vec![];
        let errors = validate(&[app_a, app_b], &test_categories());
        has_error(&errors, "is not mutual");
        has_error(&errors, "conflicts with app");
    }

    #[test]
    fn rule7_asymmetry_without_shared_binary_errors() {
        let mut app_a = make_app("app-a");
        app_a.conflicts = vec!["app-b".into()];
        let app_b = make_app("app-b");
        has_error(&validate(&[app_a, app_b], &test_categories()), "is not mutual");
    }

    #[test]
    fn rule7_unknown_app_errors() {
        let mut app = make_app("foo");
        app.conflicts = vec!["nonexistent".into()];
        has_error(
            &validate(&[app], &test_categories()),
            "conflicts references unknown app 'nonexistent'",
        );
    }

    #[test]
    fn rule7_self_reference_errors() {
        let mut app = make_app("foo");
        app.conflicts = vec!["foo".into()];
        has_error(&validate(&[app], &test_categories()), "conflicts lists itself");
    }

    #[test]
    fn rule7_identical_binary_sets_errors() {
        let (app_a, mut app_b) = make_conflicting_pair();
        app_b.binaries.pop(); // both apps now have exactly {"shared"}
        let errors = validate(&[app_a, app_b], &test_categories());
        has_error(&errors, "binary name sets are identical");
        let count = errors
            .iter()
            .filter(|e| e.contains("binary name sets are identical"))
            .count();
        assert_eq!(count, 1, "identical-sets error should be reported once per pair");
    }

    #[test]
    fn rule7_three_way_group_ok() {
        let mut app_a = make_app("app-a");
        app_a.binaries[0].name = "shared".into();
        app_a.conflicts = vec!["app-b".into(), "app-c".into()];
        let (_, mut app_b) = make_conflicting_pair();
        app_b.conflicts = vec!["app-a".into(), "app-c".into()];
        let mut app_c = make_app("app-c");
        app_c.binaries[0].name = "shared".into();
        app_c.binaries.push(AppBinaryDef {
            id:              2,
            name:            "shared-c".into(),
            version_cmdline: "--version".into(),
            is_main:         false,
        });
        app_c.conflicts = vec!["app-a".into(), "app-b".into()];
        no_errors(&validate(&[app_a, app_b, app_c], &test_categories()));
    }

    #[test]
    fn rule7_three_way_missing_one_pair_errors() {
        // app-b and app-c share "shared" but do not list each other
        let mut app_a = make_app("app-a");
        app_a.binaries[0].name = "shared".into();
        app_a.conflicts = vec!["app-b".into(), "app-c".into()];
        let (_, mut app_b) = make_conflicting_pair();
        app_b.conflicts = vec!["app-a".into()];
        let mut app_c = make_app("app-c");
        app_c.binaries[0].name = "shared".into();
        app_c.binaries.push(AppBinaryDef {
            id:              2,
            name:            "shared-c".into(),
            version_cmdline: "--version".into(),
            is_main:         false,
        });
        app_c.conflicts = vec!["app-a".into()];
        has_error(
            &validate(&[app_a, app_b, app_c], &test_categories()),
            "binary name 'shared' conflicts with app 'app-b'",
        );
    }

    // Rule 3: shell_completions uniqueness within app

    #[test]
    fn rule3_sc_self_gen_unique_key_ok() {
        let mut app = make_app("foo");
        app.shell_completions = vec![
            ShellCompletionDef {
                shell:  ShellKind::Bash,
                source: CompletionSource::SelfGenerated {
                    binary_id: 1,
                    command:   "completions bash".into(),
                },
            },
            ShellCompletionDef {
                shell:  ShellKind::Zsh,
                source: CompletionSource::SelfGenerated {
                    binary_id: 1,
                    command:   "completions zsh".into(),
                },
            },
        ];
        no_errors(&validate(&[app], &test_categories()));
    }

    #[test]
    fn rule3_sc_self_gen_duplicate_key() {
        let mut app = make_app("foo");
        app.shell_completions = vec![
            ShellCompletionDef {
                shell:  ShellKind::Bash,
                source: CompletionSource::SelfGenerated {
                    binary_id: 1,
                    command:   "completions bash".into(),
                },
            },
            ShellCompletionDef {
                shell:  ShellKind::Bash, // same binary_id + shell
                source: CompletionSource::SelfGenerated {
                    binary_id: 1,
                    command:   "completions bash".into(),
                },
            },
        ];
        has_error(
            &validate(&[app], &test_categories()),
            "duplicate self_generated completion",
        );
    }

    #[test]
    fn rule3_sc_extracted_unique_path_ok() {
        let mut app = make_app("foo");
        app.shell_completions = vec![
            ShellCompletionDef {
                shell:  ShellKind::Bash,
                source: CompletionSource::Extracted {
                    asset_id: 1,
                    path:     "foo.bash".into(),
                },
            },
            ShellCompletionDef {
                shell:  ShellKind::Zsh,
                source: CompletionSource::Extracted {
                    asset_id: 1,
                    path:     "_foo".into(),
                },
            },
        ];
        no_errors(&validate(&[app], &test_categories()));
    }

    #[test]
    fn rule3_sc_extracted_duplicate_path() {
        let mut app = make_app("foo");
        app.shell_completions = vec![
            ShellCompletionDef {
                shell:  ShellKind::Bash,
                source: CompletionSource::Extracted {
                    asset_id: 1,
                    path:     "foo.bash".into(),
                },
            },
            ShellCompletionDef {
                shell:  ShellKind::Bash,
                source: CompletionSource::Extracted {
                    asset_id: 1,
                    path:     "foo.bash".into(), // same path
                },
            },
        ];
        has_error(
            &validate(&[app], &test_categories()),
            "duplicate extracted completion path",
        );
    }

    #[test]
    fn rule3_sc_mixed_types_same_shell_ok() {
        let mut app = make_app("foo");
        // one SelfGenerated + one Extracted for bash — both are allowed
        app.shell_completions = vec![
            ShellCompletionDef {
                shell:  ShellKind::Bash,
                source: CompletionSource::SelfGenerated {
                    binary_id: 1,
                    command:   "completions bash".into(),
                },
            },
            ShellCompletionDef {
                shell:  ShellKind::Bash,
                source: CompletionSource::Extracted {
                    asset_id: 1,
                    path:     "foo.bash".into(),
                },
            },
        ];
        no_errors(&validate(&[app], &test_categories()));
    }

    // Rule 4: shell_completions global uniqueness

    #[test]
    fn rule4_sc_self_gen_global_ok() {
        let mut app_a = make_app("aaa");
        app_a.shell_completions = vec![ShellCompletionDef {
            shell:  ShellKind::Bash,
            source: CompletionSource::SelfGenerated {
                binary_id: 1,
                command:   "completions bash".into(),
            },
        }];
        let mut app_b = make_app("bbb");
        app_b.shell_completions = vec![ShellCompletionDef {
            shell:  ShellKind::Bash,
            source: CompletionSource::SelfGenerated {
                binary_id: 1,
                command:   "completions bash".into(),
            },
        }];
        // binary names differ (aaa vs bbb), so no conflict
        no_errors(&validate(&[app_a, app_b], &test_categories()));
    }

    #[test]
    fn rule4_sc_self_gen_global_conflict() {
        // Both apps define a bash SelfGenerated completion for a binary named "shared"
        let mut app_a = make_app("app-a");
        app_a.binaries[0].name = "shared".into();
        app_a.shell_completions = vec![ShellCompletionDef {
            shell:  ShellKind::Bash,
            source: CompletionSource::SelfGenerated {
                binary_id: 1,
                command:   "completions bash".into(),
            },
        }];
        let mut app_b = make_app("app-b");
        app_b.binaries[0].name = "shared".into();
        app_b.shell_completions = vec![ShellCompletionDef {
            shell:  ShellKind::Bash,
            source: CompletionSource::SelfGenerated {
                binary_id: 1,
                command:   "completions bash".into(),
            },
        }];
        // Both apps have a binary named "shared" → conflicts on both binary names and completions
        let errors = validate(&[app_a, app_b], &test_categories());
        has_error(&errors, "conflicts with app");
    }

    #[test]
    fn rule4_sc_extracted_global_ok() {
        let mut app_a = make_app("aaa");
        app_a.shell_completions = vec![ShellCompletionDef {
            shell:  ShellKind::Bash,
            source: CompletionSource::Extracted {
                asset_id: 1,
                path:     "aaa.bash".into(),
            },
        }];
        let mut app_b = make_app("bbb");
        app_b.shell_completions = vec![ShellCompletionDef {
            shell:  ShellKind::Bash,
            source: CompletionSource::Extracted {
                asset_id: 1,
                path:     "bbb.bash".into(), // different path
            },
        }];
        no_errors(&validate(&[app_a, app_b], &test_categories()));
    }

    #[test]
    fn rule4_sc_extracted_global_conflict() {
        let mut app_a = make_app("aaa");
        app_a.shell_completions = vec![ShellCompletionDef {
            shell:  ShellKind::Bash,
            source: CompletionSource::Extracted {
                asset_id: 1,
                path:     "shared.bash".into(),
            },
        }];
        let mut app_b = make_app("bbb");
        app_b.shell_completions = vec![ShellCompletionDef {
            shell:  ShellKind::Bash,
            source: CompletionSource::Extracted {
                asset_id: 1,
                path:     "shared.bash".into(), // same path
            },
        }];
        has_error(
            &validate(&[app_a, app_b], &test_categories()),
            "extracted completion path 'shared.bash' conflicts with app",
        );
    }

    // Rule 5: man_pages uniqueness within app

    #[test]
    fn rule5_mp_self_gen_unique_key_ok() {
        let mut app = make_app("foo");
        app.man_pages = vec![
            ManPageDef {
                section: 1,
                source:  CompletionSource::SelfGenerated {
                    binary_id: 1,
                    command:   "man --section 1".into(),
                },
            },
            ManPageDef {
                section: 5,
                source:  CompletionSource::SelfGenerated {
                    binary_id: 1,
                    command:   "man --section 5".into(), // different command
                },
            },
        ];
        no_errors(&validate(&[app], &test_categories()));
    }

    #[test]
    fn rule5_mp_self_gen_duplicate_key() {
        let mut app = make_app("foo");
        app.man_pages = vec![
            ManPageDef {
                section: 1,
                source:  CompletionSource::SelfGenerated {
                    binary_id: 1,
                    command:   "man".into(),
                },
            },
            ManPageDef {
                section: 1,
                source:  CompletionSource::SelfGenerated {
                    binary_id: 1,
                    command:   "man".into(), // same binary_id + command
                },
            },
        ];
        has_error(
            &validate(&[app], &test_categories()),
            "duplicate self_generated man page",
        );
    }

    #[test]
    fn rule5_mp_extracted_unique_path_ok() {
        let mut app = make_app("foo");
        app.man_pages = vec![
            ManPageDef {
                section: 1,
                source:  CompletionSource::Extracted {
                    asset_id: 1,
                    path:     "foo.1".into(),
                },
            },
            ManPageDef {
                section: 5,
                source:  CompletionSource::Extracted {
                    asset_id: 1,
                    path:     "foo.5".into(),
                },
            },
        ];
        no_errors(&validate(&[app], &test_categories()));
    }

    #[test]
    fn rule5_mp_extracted_duplicate_path() {
        let mut app = make_app("foo");
        app.man_pages = vec![
            ManPageDef {
                section: 1,
                source:  CompletionSource::Extracted {
                    asset_id: 1,
                    path:     "foo.1".into(),
                },
            },
            ManPageDef {
                section: 1,
                source:  CompletionSource::Extracted {
                    asset_id: 1,
                    path:     "foo.1".into(), // same path
                },
            },
        ];
        has_error(
            &validate(&[app], &test_categories()),
            "duplicate extracted man page path",
        );
    }

    #[test]
    fn rule5_mp_mixed_types_ok() {
        let mut app = make_app("foo");
        // one SelfGenerated + one Extracted — allowed even for the "same" man page
        app.man_pages = vec![
            ManPageDef {
                section: 1,
                source:  CompletionSource::SelfGenerated {
                    binary_id: 1,
                    command:   "man".into(),
                },
            },
            ManPageDef {
                section: 1,
                source:  CompletionSource::Extracted {
                    asset_id: 1,
                    path:     "foo.1".into(),
                },
            },
        ];
        no_errors(&validate(&[app], &test_categories()));
    }

    // Rule 6: man_pages global uniqueness

    #[test]
    fn rule6_mp_self_gen_global_ok() {
        let mut app_a = make_app("aaa");
        app_a.man_pages = vec![ManPageDef {
            section: 1,
            source:  CompletionSource::SelfGenerated {
                binary_id: 1,
                command:   "man".into(),
            },
        }];
        let mut app_b = make_app("bbb");
        app_b.man_pages = vec![ManPageDef {
            section: 1,
            source:  CompletionSource::SelfGenerated {
                binary_id: 1,
                command:   "man".into(),
            },
        }];
        // binary names differ (aaa vs bbb), so no global conflict
        no_errors(&validate(&[app_a, app_b], &test_categories()));
    }

    #[test]
    fn rule6_mp_self_gen_global_conflict() {
        let mut app_a = make_app("app-a");
        app_a.binaries[0].name = "shared".into();
        app_a.man_pages = vec![ManPageDef {
            section: 1,
            source:  CompletionSource::SelfGenerated {
                binary_id: 1,
                command:   "man".into(),
            },
        }];
        let mut app_b = make_app("app-b");
        app_b.binaries[0].name = "shared".into();
        app_b.man_pages = vec![ManPageDef {
            section: 1,
            source:  CompletionSource::SelfGenerated {
                binary_id: 1,
                command:   "man".into(),
            },
        }];
        let errors = validate(&[app_a, app_b], &test_categories());
        has_error(&errors, "conflicts with app");
    }

    #[test]
    fn rule6_mp_extracted_global_ok() {
        let mut app_a = make_app("aaa");
        app_a.man_pages = vec![ManPageDef {
            section: 1,
            source:  CompletionSource::Extracted {
                asset_id: 1,
                path:     "aaa.1".into(),
            },
        }];
        let mut app_b = make_app("bbb");
        app_b.man_pages = vec![ManPageDef {
            section: 1,
            source:  CompletionSource::Extracted {
                asset_id: 1,
                path:     "bbb.1".into(), // different path
            },
        }];
        no_errors(&validate(&[app_a, app_b], &test_categories()));
    }

    #[test]
    fn rule6_mp_extracted_global_conflict() {
        let mut app_a = make_app("aaa");
        app_a.man_pages = vec![ManPageDef {
            section: 1,
            source:  CompletionSource::Extracted {
                asset_id: 1,
                path:     "shared.1".into(),
            },
        }];
        let mut app_b = make_app("bbb");
        app_b.man_pages = vec![ManPageDef {
            section: 1,
            source:  CompletionSource::Extracted {
                asset_id: 1,
                path:     "shared.1".into(), // same path
            },
        }];
        has_error(
            &validate(&[app_a, app_b], &test_categories()),
            "extracted man page path 'shared.1' conflicts with app",
        );
    }

    // Pre-existing cross-reference rules

    #[test]
    fn existing_category_id_not_found() {
        let mut app = make_app("foo");
        app.category_id = "nonexistent".into();
        has_error(&validate(&[app], &test_categories()), "unknown category_id");
    }

    #[test]
    fn existing_unknown_binary_id_in_sc() {
        let mut app = make_app("foo");
        app.shell_completions = vec![ShellCompletionDef {
            shell:  ShellKind::Bash,
            source: CompletionSource::SelfGenerated {
                binary_id: 99, // does not exist
                command:   "completions bash".into(),
            },
        }];
        has_error(
            &validate(&[app], &test_categories()),
            "references unknown binary_id 99",
        );
    }

    #[test]
    fn existing_unknown_asset_id_in_sc() {
        let mut app = make_app("foo");
        app.shell_completions = vec![ShellCompletionDef {
            shell:  ShellKind::Bash,
            source: CompletionSource::Extracted {
                asset_id: 99, // does not exist
                path:     "foo.bash".into(),
            },
        }];
        has_error(
            &validate(&[app], &test_categories()),
            "references unknown asset_id 99",
        );
    }

    #[test]
    fn existing_unknown_binary_id_in_mp() {
        let mut app = make_app("foo");
        app.man_pages = vec![ManPageDef {
            section: 1,
            source:  CompletionSource::SelfGenerated {
                binary_id: 99, // does not exist
                command:   "man".into(),
            },
        }];
        has_error(
            &validate(&[app], &test_categories()),
            "references unknown binary_id 99",
        );
    }

    #[test]
    fn existing_unknown_asset_id_in_mp() {
        let mut app = make_app("foo");
        app.man_pages = vec![ManPageDef {
            section: 1,
            source:  CompletionSource::Extracted {
                asset_id: 99, // does not exist
                path:     "foo.1".into(),
            },
        }];
        has_error(
            &validate(&[app], &test_categories()),
            "references unknown asset_id 99",
        );
    }

    #[test]
    fn existing_duplicate_binary_ids() {
        let mut app = make_app("foo");
        app.binaries.push(AppBinaryDef {
            id:              1, // same as existing
            name:            "foox".into(),
            version_cmdline: "--version".into(),
            is_main:         false,
        });
        has_error(&validate(&[app], &test_categories()), "duplicate binary ids");
    }

    #[test]
    fn existing_duplicate_asset_ids() {
        let mut app = make_app("foo");
        app.assets.push(AppAssetDef {
            id:           1, // same as existing
            asset_type:   AssetType::Deb,
            starts_with:  None,
            contains:     None,
            not_contains: None,
            ends_with:    None,
            equals:       Some("foo.deb".into()),
        });
        has_error(&validate(&[app], &test_categories()), "duplicate asset ids");
    }

    #[test]
    fn existing_is_main_missing() {
        let mut app = make_app("foo");
        app.binaries[0].is_main = false;
        has_error(
            &validate(&[app], &test_categories()),
            "expected exactly 1 binary with is_main=true, found 0",
        );
    }

    #[test]
    fn existing_is_main_two() {
        let mut app = make_app("foo");
        app.binaries.push(AppBinaryDef {
            id:              2,
            name:            "foox".into(),
            version_cmdline: "--version".into(),
            is_main:         true, // second is_main
        });
        has_error(
            &validate(&[app], &test_categories()),
            "expected exactly 1 binary with is_main=true, found 2",
        );
    }
}
