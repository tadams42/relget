use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use anyhow::{Context, Result, anyhow, bail};

use super::app_entry::{AppEntry, CompletionSource, ShellKind};
use super::category_entry::CategoryEntry;
use super::loader;

// ===== Public types =====

#[derive(Debug, Clone)]
pub struct Registry {
    pub categories: Vec<CategoryEntry>,
    pub apps:       Vec<AppEntry>,
}

// ===== Registry =====

impl Registry {
    pub fn load() -> Result<Self> {
        Ok(Registry {
            categories: loader::load_raw_categories()?,
            apps:       loader::load_raw_apps()?,
        })
    }

    pub fn validate(&self) -> Result<()> {
        let mut errors = Self::validate_json_schemas()?;
        errors.extend(self.collect_rule_errors());
        if errors.is_empty() {
            Ok(())
        } else {
            for error in &errors {
                eprintln!("{error}");
            }
            bail!("registry validation failed with {} error(s)", errors.len())
        }
    }

    fn validate_json_schemas() -> Result<Vec<String>> {
        let mut errors: Vec<String> = Vec::new();

        let app_schema_raw = loader::RegistryFiles::get("app.schema.json")
            .ok_or_else(|| anyhow!("app.schema.json not embedded"))?;
        let cat_schema_raw = loader::RegistryFiles::get("categories.schema.json")
            .ok_or_else(|| anyhow!("categories.schema.json not embedded"))?;

        let app_schema: serde_json::Value =
            serde_json::from_slice(&app_schema_raw.data).context("parsing app.schema.json")?;
        let cat_schema: serde_json::Value = serde_json::from_slice(&cat_schema_raw.data)
            .context("parsing categories.schema.json")?;

        let app_validator = jsonschema::validator_for(&app_schema)
            .map_err(|e| anyhow!("failed to compile app.schema.json: {}", e))?;
        let cat_validator = jsonschema::validator_for(&cat_schema)
            .map_err(|e| anyhow!("failed to compile categories.schema.json: {}", e))?;

        let cat_file = loader::RegistryFiles::get("categories.jsonc")
            .ok_or_else(|| anyhow!("categories.jsonc not embedded"))?;
        let cat_value: serde_json::Value =
            loader::from_jsonc_slice(&cat_file.data, "categories.jsonc")?;
        for error in cat_validator.iter_errors(&cat_value) {
            errors.push(format!("categories.jsonc: {error}"));
        }

        let mut app_paths: Vec<String> = loader::RegistryFiles::iter()
            .map(|p| p.as_ref().to_owned())
            .filter(|p| loader::is_app_path(p))
            .collect();
        app_paths.sort();

        for path in &app_paths {
            let file = loader::RegistryFiles::get(path).expect("file was just listed");
            let value: serde_json::Value = match loader::from_jsonc_slice(&file.data, path) {
                Ok(v) => v,
                Err(e) => {
                    errors.push(format!("{path}: invalid JSON: {e}"));
                    continue;
                }
            };
            for error in app_validator.iter_errors(&value) {
                errors.push(format!("{path}: {error}"));
            }
        }

        Ok(errors)
    }

    /// Semantic validation rules operating on the parsed Registry struct.
    /// Returns a list of error strings; empty means valid.
    /// Public to allow unit testing with synthetic data.
    pub fn collect_rule_errors(&self) -> Vec<String> {
        let mut errors: Vec<String> = Vec::new();

        let category_ids: HashSet<&str> = self.categories.iter().map(|c| c.id.as_str()).collect();

        // Global uniqueness accumulators (rules 2, 4, 6)
        let mut global_binary_names: HashMap<String, String> = HashMap::new();
        let mut global_sc_gen: HashMap<(String, ShellKind), String> = HashMap::new();
        let mut global_sc_ext: HashMap<String, String> = HashMap::new();
        let mut global_mp_gen: HashMap<(String, String), String> = HashMap::new();
        let mut global_mp_ext: HashMap<String, String> = HashMap::new();

        for app in &self.apps {
            let app_id = &app.id;

            // category_id must exist in loaded categories
            if !category_ids.contains(app.category_id.as_str()) {
                errors.push(format!("{app_id}: unknown category_id '{}'", app.category_id));
            }

            let binary_ids: HashSet<u32> = app.binaries.iter().map(|b| b.id).collect();
            let asset_ids: HashSet<u32> = app.assets.iter().map(|a| a.id).collect();

            // Binary numeric id uniqueness
            if binary_ids.len() != app.binaries.len() {
                errors.push(format!("{app_id}: duplicate binary ids"));
            }

            // Rule 1: binary name uniqueness within app
            {
                let mut seen: HashSet<&str> = HashSet::new();
                for b in &app.binaries {
                    if !seen.insert(b.name.as_str()) {
                        errors.push(format!("{app_id}: duplicate binary name '{}'", b.name));
                    }
                }
            }

            // Asset numeric id uniqueness
            if asset_ids.len() != app.assets.len() {
                errors.push(format!("{app_id}: duplicate asset ids"));
            }

            // Exactly one is_main binary
            let main_count = app.binaries.iter().filter(|b| b.is_main).count();
            if main_count != 1 {
                errors.push(format!(
                    "{app_id}: expected exactly 1 binary with is_main=true, found {main_count}"
                ));
            }

            // Rule 2: binary name uniqueness globally
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

            // shell_completions: reference checks + rules 3 and 4
            let mut sc_gen_seen: HashSet<(u32, ShellKind)> = HashSet::new();
            let mut sc_ext_seen: HashSet<String> = HashSet::new();

            for sc in &app.shell_completions {
                match &sc.source {
                    CompletionSource::SelfGenerated { binary_id, .. } => {
                        if !binary_ids.contains(binary_id) {
                            errors.push(format!(
                                "{app_id}: shell_completion references unknown binary_id \
                                 {binary_id}"
                            ));
                        }
                        // Rule 3: per-app SelfGenerated uniqueness
                        if !sc_gen_seen.insert((*binary_id, sc.shell.clone())) {
                            errors.push(format!(
                                "{app_id}: duplicate self_generated completion for \
                                 binary_id={binary_id} shell={:?}",
                                sc.shell
                            ));
                        }
                        // Rule 4: global SelfGenerated uniqueness
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
                        // Rule 3: per-app Extracted uniqueness
                        if !sc_ext_seen.insert(path.clone()) {
                            errors.push(format!(
                                "{app_id}: duplicate extracted completion path '{path}'"
                            ));
                        }
                        // Rule 4: global Extracted uniqueness
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

            // man_pages: reference checks + rules 5 and 6
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
                        // Rule 5: per-app SelfGenerated uniqueness
                        if !mp_gen_seen.insert((*binary_id, command.clone())) {
                            errors.push(format!(
                                "{app_id}: duplicate self_generated man page for \
                                 binary_id={binary_id} command='{command}'"
                            ));
                        }
                        // Rule 6: global SelfGenerated uniqueness
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
                        // Rule 5: per-app Extracted uniqueness
                        if !mp_ext_seen.insert(path.clone()) {
                            errors.push(format!(
                                "{app_id}: duplicate extracted man page path '{path}'"
                            ));
                        }
                        // Rule 6: global Extracted uniqueness
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
}

// ===== Static global accessor =====

static REGISTRY: OnceLock<Registry> = OnceLock::new();

impl Registry {
    pub fn global() -> &'static Self {
        REGISTRY.get_or_init(|| Self::load().expect("failed to load registry"))
    }

    pub fn entries(&self) -> &[AppEntry] { &self.apps }

    pub fn identifiers(&self) -> Vec<&str> {
        let mut ids: Vec<&str> = self.apps.iter().map(|a| a.id.as_str()).collect();
        ids.sort_unstable();
        ids
    }

    pub fn categories(&self) -> &[CategoryEntry] { &self.categories }

    pub fn doctor(&self, offline: bool) -> Result<()> { super::doctor::doctor(&self.apps, offline) }
}

// ===== Tests =====

#[cfg(test)]
mod tests {
    use super::Registry;
    use crate::registry::{
        AppAssetDef, AppBinaryDef, AppEntry, AssetType, CategoryEntry, CompletionSource,
        ManPageDef, ShellCompletionDef, ShellKind,
    };

    // ===== Helpers =====

    fn make_registry(apps: Vec<AppEntry>) -> Registry {
        Registry {
            categories: vec![CategoryEntry {
                id:          "test".into(),
                title:       "Test".into(),
                description: None,
            }],
            apps,
        }
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
        no_errors(&make_registry(vec![app]).collect_rule_errors());
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
        has_error(
            &make_registry(vec![app]).collect_rule_errors(),
            "duplicate binary name",
        );
    }

    // Rule 2: binary name uniqueness globally

    #[test]
    fn rule2_binary_names_unique_globally_ok() {
        let app_a = make_app("aaa");
        let app_b = make_app("bbb");
        no_errors(&make_registry(vec![app_a, app_b]).collect_rule_errors());
    }

    #[test]
    fn rule2_binary_names_conflict_globally() {
        let app_a = make_app("shared");
        let mut app_b = make_app("other");
        // binary in app_b named "shared" conflicts with app_a's binary
        app_b.binaries[0].name = "shared".into();
        has_error(
            &make_registry(vec![app_a, app_b]).collect_rule_errors(),
            "conflicts with app",
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
        no_errors(&make_registry(vec![app]).collect_rule_errors());
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
            &make_registry(vec![app]).collect_rule_errors(),
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
        no_errors(&make_registry(vec![app]).collect_rule_errors());
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
            &make_registry(vec![app]).collect_rule_errors(),
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
        no_errors(&make_registry(vec![app]).collect_rule_errors());
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
        no_errors(&make_registry(vec![app_a, app_b]).collect_rule_errors());
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
        let errors = make_registry(vec![app_a, app_b]).collect_rule_errors();
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
        no_errors(&make_registry(vec![app_a, app_b]).collect_rule_errors());
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
            &make_registry(vec![app_a, app_b]).collect_rule_errors(),
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
        no_errors(&make_registry(vec![app]).collect_rule_errors());
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
            &make_registry(vec![app]).collect_rule_errors(),
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
        no_errors(&make_registry(vec![app]).collect_rule_errors());
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
            &make_registry(vec![app]).collect_rule_errors(),
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
        no_errors(&make_registry(vec![app]).collect_rule_errors());
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
        no_errors(&make_registry(vec![app_a, app_b]).collect_rule_errors());
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
        let errors = make_registry(vec![app_a, app_b]).collect_rule_errors();
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
        no_errors(&make_registry(vec![app_a, app_b]).collect_rule_errors());
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
            &make_registry(vec![app_a, app_b]).collect_rule_errors(),
            "extracted man page path 'shared.1' conflicts with app",
        );
    }

    // Pre-existing cross-reference rules

    #[test]
    fn existing_category_id_not_found() {
        let mut app = make_app("foo");
        app.category_id = "nonexistent".into();
        has_error(&make_registry(vec![app]).collect_rule_errors(), "unknown category_id");
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
            &make_registry(vec![app]).collect_rule_errors(),
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
            &make_registry(vec![app]).collect_rule_errors(),
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
            &make_registry(vec![app]).collect_rule_errors(),
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
            &make_registry(vec![app]).collect_rule_errors(),
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
        has_error(
            &make_registry(vec![app]).collect_rule_errors(),
            "duplicate binary ids",
        );
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
        has_error(&make_registry(vec![app]).collect_rule_errors(), "duplicate asset ids");
    }

    #[test]
    fn existing_is_main_missing() {
        let mut app = make_app("foo");
        app.binaries[0].is_main = false;
        has_error(
            &make_registry(vec![app]).collect_rule_errors(),
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
            &make_registry(vec![app]).collect_rule_errors(),
            "expected exactly 1 binary with is_main=true, found 2",
        );
    }

    // ===== Cross-module invariant tests =====

    #[test]
    fn registry_ids_are_unique() {
        use std::collections::HashSet;
        let ids: Vec<_> = Registry::global()
            .entries()
            .iter()
            .map(|a| a.id.as_str())
            .collect();
        let unique: HashSet<_> = ids.iter().copied().collect();
        assert_eq!(ids.len(), unique.len(), "duplicate app ids in registry");
    }

    #[test]
    fn identifiers_is_sorted() {
        let ids = Registry::global().identifiers();
        let sorted = ids.windows(2).all(|w| w[0] <= w[1]);
        assert!(sorted, "identifiers() is not sorted");
    }

    #[test]
    fn registry_exe_names_match_app_trait() {
        use crate::App;
        for app in Registry::global().entries() {
            let instance = App::from_id(&app.id, None, None, None, true)
                .unwrap_or_else(|| panic!("from_id returned None for id '{}'", app.id));
            assert_eq!(
                app.main_exe_name(),
                instance.exe_name(),
                "registry main_exe_name mismatch for id '{}': registry='{}' instance='{}'",
                app.id,
                app.main_exe_name(),
                instance.exe_name()
            );
        }
    }

    #[test]
    fn all_apps_have_factory_entry() {
        use crate::App;
        for app in Registry::global().entries() {
            assert!(
                App::from_id(&app.id, None, None, None, true).is_some(),
                "from_id returned None for registry id '{}'",
                app.id
            );
        }
    }

    #[test]
    fn all_apps_have_binary_descriptor() {
        use crate::App;
        for app in Registry::global().entries() {
            let instance = App::from_id(&app.id, None, None, None, true)
                .unwrap_or_else(|| panic!("from_id returned None for id '{}'", app.id));
            assert!(
                instance.assets().binary.is_some(),
                "app '{}' has no primary binary descriptor in assets()",
                app.id
            );
        }
    }
}
