mod doctor;

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use anyhow::{Context, Result, anyhow, bail};
use rust_embed::RustEmbed;
use serde::Deserialize;

#[derive(RustEmbed)]
#[folder = "src/registry/"]
#[include = "**/*.json"]
struct RegistryFiles;

// ===== Public types =====

#[derive(Debug, Clone)]
pub struct Registry {
    pub categories: Vec<CategoryEntry>,
    pub apps:       Vec<AppEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CategoryEntry {
    pub id:          String,
    pub title:       String,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AppEntry {
    pub id:                String,
    pub category_id:       String,
    pub description:       Option<String>,
    pub url:               String,
    pub has_musl:          bool,
    pub binaries:          Vec<AppBinaryDef>,
    pub assets:            Vec<AppAssetDef>,
    pub shell_completions: Vec<ShellCompletionDef>,
    pub man_pages:         Vec<ManPageDef>,
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
    pub id:          u32,
    pub asset_type:  AssetType,
    pub starts_with: Option<String>,
    pub contains:    Option<String>,
    pub ends_with:   Option<String>,
    pub equals:      Option<String>,
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
    id:          u32,
    #[serde(rename = "type")]
    asset_type:  String,
    starts_with: Option<String>,
    contains:    Option<String>,
    ends_with:   Option<String>,
    equals:      Option<String>,
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

fn is_app_path(path: &str) -> bool {
    let mut parts = path.splitn(2, '/');
    match (parts.next(), parts.next()) {
        (Some(dir), Some(file)) => {
            dir.len() == 1
                && dir.chars().all(|c| c.is_ascii_lowercase())
                && file.ends_with(".json")
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

// ===== Registry =====

impl Registry {
    pub fn load() -> Result<Self> {
        let cat_file = RegistryFiles::get("categories.json")
            .ok_or_else(|| anyhow!("categories.json not embedded"))?;
        let raw_cats: RawCategories =
            serde_json::from_slice(&cat_file.data).context("parsing categories.json")?;

        let mut apps = Vec::new();
        let mut paths: Vec<String> = RegistryFiles::iter()
            .map(|p| p.as_ref().to_owned())
            .filter(|p| is_app_path(p))
            .collect();
        paths.sort();

        for path in &paths {
            let file = RegistryFiles::get(path).expect("file was just listed");
            let raw: RawApp =
                serde_json::from_slice(&file.data).with_context(|| format!("parsing {}", path))?;
            apps.push(convert_app(raw, path).with_context(|| format!("converting {}", path))?);
        }

        Ok(Registry {
            categories: raw_cats.categories,
            apps,
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

        let app_schema_raw = RegistryFiles::get("app.schema.json")
            .ok_or_else(|| anyhow!("app.schema.json not embedded"))?;
        let cat_schema_raw = RegistryFiles::get("categories.schema.json")
            .ok_or_else(|| anyhow!("categories.schema.json not embedded"))?;

        let app_schema: serde_json::Value =
            serde_json::from_slice(&app_schema_raw.data).context("parsing app.schema.json")?;
        let cat_schema: serde_json::Value = serde_json::from_slice(&cat_schema_raw.data)
            .context("parsing categories.schema.json")?;

        let app_validator = jsonschema::validator_for(&app_schema)
            .map_err(|e| anyhow!("failed to compile app.schema.json: {}", e))?;
        let cat_validator = jsonschema::validator_for(&cat_schema)
            .map_err(|e| anyhow!("failed to compile categories.schema.json: {}", e))?;

        let cat_file = RegistryFiles::get("categories.json")
            .ok_or_else(|| anyhow!("categories.json not embedded"))?;
        let cat_value: serde_json::Value =
            serde_json::from_slice(&cat_file.data).context("parsing categories.json")?;
        for error in cat_validator.iter_errors(&cat_value) {
            errors.push(format!("categories.json: {error}"));
        }

        let mut app_paths: Vec<String> = RegistryFiles::iter()
            .map(|p| p.as_ref().to_owned())
            .filter(|p| is_app_path(p))
            .collect();
        app_paths.sort();

        for path in &app_paths {
            let file = RegistryFiles::get(path).expect("file was just listed");
            let value: serde_json::Value = match serde_json::from_slice(&file.data) {
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

    pub fn doctor(&self, offline: bool) -> Result<()> { doctor::doctor(&self.apps, offline) }
}

// ===== RegistryApp helpers =====

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
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

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
            id:                id.into(),
            category_id:       "test".into(),
            description:       None,
            url:               "https://example.com".into(),
            has_musl:          false,
            binaries:          vec![AppBinaryDef {
                id:              1,
                name:            id.into(),
                version_cmdline: "--version".into(),
                is_main:         true,
            }],
            assets:            vec![AppAssetDef {
                id:          1,
                asset_type:  AssetType::Archive,
                starts_with: None,
                contains:    None,
                ends_with:   None,
                equals:      Some("foo.tar.gz".into()),
            }],
            shell_completions: vec![],
            man_pages:         vec![],
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
            id:          1, // same as existing
            asset_type:  AssetType::Deb,
            starts_with: None,
            contains:    None,
            ends_with:   None,
            equals:      Some("foo.deb".into()),
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
        use crate::create_app;
        for app in Registry::global().entries() {
            let instance = create_app(&app.id, None, None, None, true)
                .unwrap_or_else(|| panic!("create_app returned None for id '{}'", app.id));
            assert_eq!(
                app.main_exe_name(),
                instance.exe_name(),
                "registry main_exe_name mismatch for id '{}': registry='{}' trait='{}'",
                app.id,
                app.main_exe_name(),
                instance.exe_name()
            );
        }
    }

    #[test]
    fn all_apps_have_factory_entry() {
        use crate::create_app;
        for app in Registry::global().entries() {
            assert!(
                create_app(&app.id, None, None, None, true).is_some(),
                "create_app returned None for registry id '{}'",
                app.id
            );
        }
    }

    #[test]
    fn all_apps_have_binary_descriptor() {
        use crate::create_app;
        for app in Registry::global().entries() {
            let instance = create_app(&app.id, None, None, None, true)
                .unwrap_or_else(|| panic!("create_app returned None for id '{}'", app.id));
            assert!(
                instance.assets().binary.is_some(),
                "app '{}' has no primary binary descriptor in assets()",
                app.id
            );
        }
    }
}
