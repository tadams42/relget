use anyhow::{Context, Result};

use registry_core::{AppEntry, CategoryEntry};

static REGISTRY_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/registry.bin"));

pub(super) fn load_registry() -> Result<(Vec<CategoryEntry>, Vec<AppEntry>)> {
    postcard::from_bytes(REGISTRY_BYTES).context("deserializing embedded registry")
}

// ===== Tests =====

#[cfg(test)]
mod tests {
    use serde_json::json;

    fn app_schema() -> serde_json::Value {
        let data =
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/registry/app.schema.json"));
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
}
