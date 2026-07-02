use std::sync::OnceLock;

use anyhow::{Context, Result};

use super::types::{AppEntry, CategoryEntry};

static REGISTRY_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/registry.bin"));

// ===== Public types =====

#[derive(Debug, Clone)]
pub struct Registry {
    pub categories: Vec<CategoryEntry>,
    pub apps:       Vec<AppEntry>,
}

// ===== Registry =====

impl Registry {
    pub fn load() -> Result<Self> {
        let (categories, apps) =
            postcard::from_bytes(REGISTRY_BYTES).context("deserializing embedded registry")?;
        Ok(Registry { categories, apps })
    }

    /// Semantic validation rules operating on the parsed Registry struct.
    /// Returns a list of error strings; empty means valid.
    /// Public to allow unit testing with synthetic data.
    pub fn collect_rule_errors(&self) -> Vec<String> {
        super::types::validate(&self.apps, &self.categories)
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
    use serde_json::json;

    use super::Registry;
    use crate::registry::types::{
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

    // ===== JSON Schema helpers =====

    fn app_schema() -> serde_json::Value {
        let data = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/registry/schema/app.schema.json"
        ));
        serde_json::from_slice(data).unwrap()
    }

    fn app_validator() -> jsonschema::Validator {
        let schema = app_schema();
        jsonschema::validator_for(&schema).unwrap()
    }

    fn minimal_app() -> serde_json::Value {
        json!({
            "id": "foo",
            "category_id": "test",
            "url": "https://github.com/foo/bar",
            "binaries": [
                { "id": 1, "name": "foo", "version_cmdline": "--version", "is_main": true }
            ],
            "assets": [
                { "id": 1, "type": "archive", "equals": "foo.tar.gz" }
            ]
        })
    }

    // ===== JSON Schema tests =====

    #[test]
    fn schema_app_minimal_valid() {
        assert!(app_validator().is_valid(&minimal_app()));
    }

    #[test]
    fn schema_app_missing_id() {
        let mut app = minimal_app();
        app.as_object_mut().unwrap().remove("id");
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_app_missing_category_id() {
        let mut app = minimal_app();
        app.as_object_mut().unwrap().remove("category_id");
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_app_missing_url() {
        let mut app = minimal_app();
        app.as_object_mut().unwrap().remove("url");
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_app_missing_binaries() {
        let mut app = minimal_app();
        app.as_object_mut().unwrap().remove("binaries");
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_app_missing_assets() {
        let mut app = minimal_app();
        app.as_object_mut().unwrap().remove("assets");
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_app_empty_id() {
        let mut app = minimal_app();
        app["id"] = json!("");
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_app_empty_category_id() {
        let mut app = minimal_app();
        app["category_id"] = json!("");
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_app_empty_url() {
        let mut app = minimal_app();
        app["url"] = json!("");
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_app_unknown_top_level_property() {
        let mut app = minimal_app();
        app["unknown_key"] = json!("value");
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_binaries_empty_array() {
        let mut app = minimal_app();
        app["binaries"] = json!([]);
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_binaries_missing_id() {
        let mut app = minimal_app();
        app["binaries"] = json!([{ "name": "foo", "version_cmdline": "--version" }]);
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_binaries_missing_name() {
        let mut app = minimal_app();
        app["binaries"] = json!([{ "id": 1, "version_cmdline": "--version" }]);
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_binaries_missing_version_cmdline() {
        let mut app = minimal_app();
        app["binaries"] = json!([{ "id": 1, "name": "foo" }]);
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_binaries_id_zero() {
        let mut app = minimal_app();
        app["binaries"] = json!([{ "id": 0, "name": "foo", "version_cmdline": "--version" }]);
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_binaries_empty_name() {
        let mut app = minimal_app();
        app["binaries"] = json!([{ "id": 1, "name": "", "version_cmdline": "--version" }]);
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_binaries_empty_version_cmdline() {
        let mut app = minimal_app();
        app["binaries"] = json!([{ "id": 1, "name": "foo", "version_cmdline": "" }]);
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_binaries_is_main_optional() {
        let mut app = minimal_app();
        app["binaries"] = json!([{ "id": 1, "name": "foo", "version_cmdline": "--version" }]);
        assert!(app_validator().is_valid(&app));
    }

    #[test]
    fn schema_assets_empty_array() {
        let mut app = minimal_app();
        app["assets"] = json!([]);
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_assets_missing_id() {
        let mut app = minimal_app();
        app["assets"] = json!([{ "type": "archive", "equals": "foo.tar.gz" }]);
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_assets_missing_type() {
        let mut app = minimal_app();
        app["assets"] = json!([{ "id": 1, "equals": "foo.tar.gz" }]);
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_assets_unknown_type() {
        let mut app = minimal_app();
        app["assets"] = json!([{ "id": 1, "type": "rpm", "equals": "foo.rpm" }]);
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_assets_no_match_condition() {
        let mut app = minimal_app();
        app["assets"] = json!([{ "id": 1, "type": "archive" }]);
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_assets_starts_with_alone_ok() {
        let mut app = minimal_app();
        app["assets"] = json!([{ "id": 1, "type": "archive", "starts_with": "foo-" }]);
        assert!(app_validator().is_valid(&app));
    }

    #[test]
    fn schema_assets_contains_alone_ok() {
        let mut app = minimal_app();
        app["assets"] = json!([{ "id": 1, "type": "archive", "contains": "x86_64" }]);
        assert!(app_validator().is_valid(&app));
    }

    #[test]
    fn schema_assets_ends_with_alone_ok() {
        let mut app = minimal_app();
        app["assets"] = json!([{ "id": 1, "type": "archive", "ends_with": ".tar.gz" }]);
        assert!(app_validator().is_valid(&app));
    }

    #[test]
    fn schema_assets_equals_alone_ok() {
        let mut app = minimal_app();
        app["assets"] = json!([{ "id": 1, "type": "archive", "equals": "foo-x86_64.tar.gz" }]);
        assert!(app_validator().is_valid(&app));
    }

    #[test]
    fn schema_assets_empty_starts_with() {
        let mut app = minimal_app();
        app["assets"] = json!([{ "id": 1, "type": "archive", "starts_with": "" }]);
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_assets_empty_contains() {
        let mut app = minimal_app();
        app["assets"] = json!([{ "id": 1, "type": "archive", "contains": "" }]);
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_assets_empty_ends_with() {
        let mut app = minimal_app();
        app["assets"] = json!([{ "id": 1, "type": "archive", "ends_with": "" }]);
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_assets_empty_equals() {
        let mut app = minimal_app();
        app["assets"] = json!([{ "id": 1, "type": "archive", "equals": "" }]);
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_shell_completions_unknown_shell() {
        let mut app = minimal_app();
        app["shell_completions"] = json!([{
            "shell": "powershell",
            "self_generated": { "binary_id": 1, "command": "completions powershell" }
        }]);
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_shell_completions_both_sources() {
        let mut app = minimal_app();
        app["shell_completions"] = json!([{
            "shell": "bash",
            "self_generated": { "binary_id": 1, "command": "completions bash" },
            "extracted": { "asset_id": 1, "path": "foo.bash" }
        }]);
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_shell_completions_no_source() {
        let mut app = minimal_app();
        app["shell_completions"] = json!([{ "shell": "bash" }]);
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_shell_completions_self_gen_binary_id_zero() {
        let mut app = minimal_app();
        app["shell_completions"] = json!([{
            "shell": "bash",
            "self_generated": { "binary_id": 0, "command": "completions bash" }
        }]);
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_shell_completions_self_gen_empty_command() {
        let mut app = minimal_app();
        app["shell_completions"] = json!([{
            "shell": "bash",
            "self_generated": { "binary_id": 1, "command": "" }
        }]);
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_shell_completions_extracted_asset_id_zero() {
        let mut app = minimal_app();
        app["shell_completions"] = json!([{
            "shell": "bash",
            "extracted": { "asset_id": 0, "path": "foo.bash" }
        }]);
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_shell_completions_extracted_empty_path() {
        let mut app = minimal_app();
        app["shell_completions"] = json!([{
            "shell": "bash",
            "extracted": { "asset_id": 1, "path": "" }
        }]);
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_shell_completions_valid_self_generated() {
        let mut app = minimal_app();
        app["shell_completions"] = json!([{
            "shell": "zsh",
            "self_generated": { "binary_id": 1, "command": "completions zsh" }
        }]);
        assert!(app_validator().is_valid(&app));
    }

    #[test]
    fn schema_shell_completions_valid_extracted() {
        let mut app = minimal_app();
        app["shell_completions"] = json!([{
            "shell": "fish",
            "extracted": { "asset_id": 1, "path": "foo.fish" }
        }]);
        assert!(app_validator().is_valid(&app));
    }

    #[test]
    fn schema_shell_completions_optional() {
        let app = minimal_app();
        assert!(app_validator().is_valid(&app));
    }

    #[test]
    fn schema_man_pages_missing_section() {
        let mut app = minimal_app();
        app["man_pages"] = json!([{
            "extracted": { "asset_id": 1, "path": "foo.1" }
        }]);
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_man_pages_section_zero() {
        let mut app = minimal_app();
        app["man_pages"] = json!([{
            "section": 0,
            "extracted": { "asset_id": 1, "path": "foo.1" }
        }]);
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_man_pages_section_nine() {
        let mut app = minimal_app();
        app["man_pages"] = json!([{
            "section": 9,
            "extracted": { "asset_id": 1, "path": "foo.1" }
        }]);
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_man_pages_section_one_ok() {
        let mut app = minimal_app();
        app["man_pages"] = json!([{
            "section": 1,
            "extracted": { "asset_id": 1, "path": "foo.1" }
        }]);
        assert!(app_validator().is_valid(&app));
    }

    #[test]
    fn schema_man_pages_section_eight_ok() {
        let mut app = minimal_app();
        app["man_pages"] = json!([{
            "section": 8,
            "extracted": { "asset_id": 1, "path": "foo.8" }
        }]);
        assert!(app_validator().is_valid(&app));
    }

    #[test]
    fn schema_man_pages_both_sources() {
        let mut app = minimal_app();
        app["man_pages"] = json!([{
            "section": 1,
            "self_generated": { "binary_id": 1, "command": "man" },
            "extracted": { "asset_id": 1, "path": "foo.1" }
        }]);
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_man_pages_no_source() {
        let mut app = minimal_app();
        app["man_pages"] = json!([{ "section": 1 }]);
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_app_complex_valid() {
        let app = json!({
            "id": "multi",
            "category_id": "test",
            "description": "A multi-binary app",
            "url": "https://github.com/example/multi",
            "binaries": [
                { "id": 1, "name": "multi", "version_cmdline": "--version", "is_main": true },
                { "id": 2, "name": "multix", "version_cmdline": "version" }
            ],
            "assets": [
                { "id": 1, "type": "archive", "starts_with": "multi-", "ends_with": "-musl.tar.gz" },
                { "id": 2, "type": "deb", "equals": "multi.deb" }
            ],
            "shell_completions": [
                { "shell": "bash", "self_generated": { "binary_id": 1, "command": "completions bash" } },
                { "shell": "zsh",  "extracted": { "asset_id": 2, "path": "_multi" } },
                { "shell": "fish", "self_generated": { "binary_id": 2, "command": "completions fish" } }
            ],
            "man_pages": [
                { "section": 1, "self_generated": { "binary_id": 1, "command": "man --generate" } },
                { "section": 5, "extracted": { "asset_id": 1, "path": "multi.5" } }
            ]
        });
        assert!(app_validator().is_valid(&app));
    }

    #[test]
    fn schema_released_version_parse_tag_starts_with_ok() {
        let mut app = minimal_app();
        app["released_version_parse"] = json!({ "tag_starts_with": "v" });
        assert!(app_validator().is_valid(&app));
    }

    #[test]
    fn schema_released_version_parse_try_in_body_ok() {
        let mut app = minimal_app();
        app["released_version_parse"] = json!({ "try_in_body": true });
        assert!(app_validator().is_valid(&app));
    }

    #[test]
    fn schema_released_version_parse_full_ok() {
        let mut app = minimal_app();
        app["released_version_parse"] = json!({ "tag_starts_with": "v", "try_in_body": true });
        assert!(app_validator().is_valid(&app));
    }

    #[test]
    fn schema_released_version_parse_empty_tag_starts_with_rejected() {
        let mut app = minimal_app();
        app["released_version_parse"] = json!({ "tag_starts_with": "" });
        assert!(!app_validator().is_valid(&app));
    }

    #[test]
    fn schema_released_version_parse_unknown_key_rejected() {
        let mut app = minimal_app();
        app["released_version_parse"] = json!({ "unknown": "x" });
        assert!(!app_validator().is_valid(&app));
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
