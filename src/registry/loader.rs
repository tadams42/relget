use anyhow::{Context, Result, anyhow, bail};
use json_comments::StripComments;
use rust_embed::RustEmbed;
use serde::Deserialize;

use super::app_entry::{
    AppAssetDef, AppBinaryDef, AppEntry, AssetType, CompletionSource, ManPageDef,
    ShellCompletionDef, ShellKind,
};
use super::category_entry::CategoryEntry;

#[derive(RustEmbed)]
#[folder = "src/registry/"]
#[include = "**/*.json"]
#[include = "**/*.jsonc"]
pub(super) struct RegistryFiles;

// ===== Private raw deserialization types =====

#[derive(Deserialize)]
struct RawCategories {
    categories: Vec<CategoryEntry>,
}

#[derive(Deserialize)]
struct RawApp {
    id:                String,
    category_id:       String,
    description:       Option<String>,
    url:               String,
    has_musl:          bool,
    binaries:          Vec<RawBinaryDef>,
    assets:            Vec<RawAssetDef>,
    #[serde(default)]
    shell_completions: Vec<RawShellCompletionDef>,
    #[serde(default)]
    man_pages:         Vec<RawManPageDef>,
}

#[derive(Deserialize)]
struct RawBinaryDef {
    id:              u32,
    name:            String,
    version_cmdline: String,
    #[serde(default)]
    is_main:         bool,
}

#[derive(Deserialize)]
struct RawAssetDef {
    id:           u32,
    #[serde(rename = "type")]
    asset_type:   String,
    starts_with:  Option<String>,
    contains:     Option<String>,
    not_contains: Option<String>,
    ends_with:    Option<String>,
    equals:       Option<String>,
}

#[derive(Deserialize)]
struct RawShellCompletionDef {
    shell:          String,
    self_generated: Option<RawSelfGeneratedDef>,
    extracted:      Option<RawExtractedDef>,
}

#[derive(Deserialize)]
struct RawManPageDef {
    section:        u8,
    self_generated: Option<RawSelfGeneratedDef>,
    extracted:      Option<RawExtractedDef>,
}

#[derive(Deserialize)]
struct RawSelfGeneratedDef {
    binary_id: u32,
    command:   String,
}

#[derive(Deserialize)]
struct RawExtractedDef {
    asset_id: u32,
    path:     String,
}

// ===== Helpers =====

pub(super) fn is_app_path(path: &str) -> bool {
    let mut parts = path.splitn(2, '/');
    match (parts.next(), parts.next()) {
        (Some(dir), Some(file)) => {
            dir.len() == 1
                && dir.chars().all(|c| c.is_ascii_lowercase())
                && file.ends_with(".jsonc")
                && !file.contains('/')
        }
        _ => false,
    }
}

fn parse_asset_type(s: &str, ctx: &str) -> Result<AssetType> {
    match s {
        "archive" => Ok(AssetType::Archive),
        "deb" => Ok(AssetType::Deb),
        "binary" => Ok(AssetType::Binary),
        other => bail!("{}: unknown asset type '{}'", ctx, other),
    }
}

fn parse_shell_kind(s: &str, ctx: &str) -> Result<ShellKind> {
    match s {
        "bash" => Ok(ShellKind::Bash),
        "zsh" => Ok(ShellKind::Zsh),
        "fish" => Ok(ShellKind::Fish),
        other => bail!("{}: unknown shell '{}'", ctx, other),
    }
}

fn parse_completion_source(
    self_generated: Option<RawSelfGeneratedDef>, extracted: Option<RawExtractedDef>, ctx: &str,
) -> Result<CompletionSource> {
    match (self_generated, extracted) {
        (Some(sg), None) => {
            Ok(CompletionSource::SelfGenerated {
                binary_id: sg.binary_id,
                command:   sg.command,
            })
        }
        (None, Some(ex)) => {
            Ok(CompletionSource::Extracted {
                asset_id: ex.asset_id,
                path:     ex.path,
            })
        }
        (Some(_), Some(_)) => bail!("{}: both self_generated and extracted specified", ctx),
        (None, None) => bail!("{}: neither self_generated nor extracted specified", ctx),
    }
}

fn convert_app(raw: RawApp, path: &str) -> Result<AppEntry> {
    let binaries = raw
        .binaries
        .into_iter()
        .map(|b| {
            AppBinaryDef {
                id:              b.id,
                name:            b.name,
                version_cmdline: b.version_cmdline,
                is_main:         b.is_main,
            }
        })
        .collect();

    let assets = raw
        .assets
        .into_iter()
        .map(|a| {
            let asset_type = parse_asset_type(&a.asset_type, path)?;
            Ok(AppAssetDef {
                id: a.id,
                asset_type,
                starts_with: a.starts_with,
                contains: a.contains,
                not_contains: a.not_contains,
                ends_with: a.ends_with,
                equals: a.equals,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let shell_completions = raw
        .shell_completions
        .into_iter()
        .map(|sc| {
            let shell = parse_shell_kind(&sc.shell, path)?;
            let source = parse_completion_source(sc.self_generated, sc.extracted, path)?;
            Ok(ShellCompletionDef { shell, source })
        })
        .collect::<Result<Vec<_>>>()?;

    let man_pages = raw
        .man_pages
        .into_iter()
        .map(|mp| {
            let source = parse_completion_source(mp.self_generated, mp.extracted, path)?;
            Ok(ManPageDef {
                section: mp.section,
                source,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(AppEntry {
        id: raw.id,
        category_id: raw.category_id,
        description: raw.description,
        url: raw.url,
        has_musl: raw.has_musl,
        binaries,
        assets,
        shell_completions,
        man_pages,
    })
}

// ===== Public loading functions =====

fn from_jsonc_slice<T: serde::de::DeserializeOwned>(data: &[u8], ctx: &str) -> Result<T> {
    let reader = StripComments::new(data);
    serde_json::from_reader(reader).with_context(|| format!("parsing {}", ctx))
}

pub(super) fn load_raw_categories() -> Result<Vec<CategoryEntry>> {
    let cat_file = RegistryFiles::get("categories.jsonc")
        .ok_or_else(|| anyhow!("categories.jsonc not embedded"))?;
    let raw_cats: RawCategories = from_jsonc_slice(&cat_file.data, "categories.jsonc")?;
    Ok(raw_cats.categories)
}

pub(super) fn load_raw_apps() -> Result<Vec<AppEntry>> {
    let mut paths: Vec<String> = RegistryFiles::iter()
        .map(|p| p.as_ref().to_owned())
        .filter(|p| is_app_path(p))
        .collect();
    paths.sort();

    let mut apps = Vec::new();
    for path in &paths {
        let file = RegistryFiles::get(path).expect("file was just listed");
        let raw: RawApp = from_jsonc_slice(&file.data, path)?;
        apps.push(convert_app(raw, path).with_context(|| format!("converting {}", path))?);
    }
    Ok(apps)
}

// ===== Tests =====

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::RegistryFiles;

    // ===== Helpers =====

    fn app_validator() -> jsonschema::Validator {
        let raw = RegistryFiles::get("app.schema.json").unwrap();
        let schema: serde_json::Value = serde_json::from_slice(&raw.data).unwrap();
        jsonschema::validator_for(&schema).unwrap()
    }

    fn minimal_app() -> serde_json::Value {
        json!({
            "id": "foo",
            "category_id": "test",
            "url": "https://github.com/foo/bar",
            "has_musl": false,
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
    fn schema_app_missing_has_musl() {
        let mut app = minimal_app();
        app.as_object_mut().unwrap().remove("has_musl");
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
            "has_musl": true,
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
}
