use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use json_comments::StripComments;
use serde::{Deserialize, Serialize};

// ===== Shared types (must match src/registry/app_entry.rs and category_entry.rs) =====

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReleasedVersionParseDef {
    tag_starts_with: Option<String>,
    try_in_body:     bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppEntry {
    id:                     String,
    category_id:            String,
    description:            Option<String>,
    url:                    String,
    binaries:               Vec<AppBinaryDef>,
    assets:                 Vec<AppAssetDef>,
    shell_completions:      Vec<ShellCompletionDef>,
    man_pages:              Vec<ManPageDef>,
    released_version_parse: Option<ReleasedVersionParseDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppBinaryDef {
    id:              u32,
    name:            String,
    version_cmdline: String,
    is_main:         bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppAssetDef {
    id:           u32,
    asset_type:   AssetType,
    starts_with:  Option<String>,
    contains:     Option<String>,
    not_contains: Option<String>,
    ends_with:    Option<String>,
    equals:       Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum AssetType {
    Archive,
    Deb,
    Binary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ShellCompletionDef {
    shell:  ShellKind,
    source: CompletionSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum ShellKind {
    Bash,
    Zsh,
    Fish,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum CompletionSource {
    SelfGenerated { binary_id: u32, command: String },
    Extracted { asset_id: u32, path: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ManPageDef {
    section: u8,
    source:  CompletionSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CategoryEntry {
    id:          String,
    title:       String,
    description: Option<String>,
}

// ===== Raw deserialization types =====

#[derive(Deserialize)]
struct RawCategories {
    categories: Vec<CategoryEntry>,
}

#[derive(Deserialize)]
struct RawReleasedVersionParseDef {
    tag_starts_with: Option<String>,
    #[serde(default)]
    try_in_body:     bool,
}

#[derive(Deserialize)]
struct RawApp {
    id:                     String,
    category_id:            String,
    description:            Option<String>,
    url:                    String,
    binaries:               Vec<RawBinaryDef>,
    assets:                 Vec<RawAssetDef>,
    #[serde(default)]
    shell_completions:      Vec<RawShellCompletionDef>,
    #[serde(default)]
    man_pages:              Vec<RawManPageDef>,
    released_version_parse: Option<RawReleasedVersionParseDef>,
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

fn from_jsonc_slice<T: serde::de::DeserializeOwned>(data: &[u8], ctx: &str) -> Result<T> {
    let reader = StripComments::new(data);
    serde_json::from_reader(reader).with_context(|| format!("parsing {ctx}"))
}

fn parse_asset_type(s: &str, ctx: &str) -> Result<AssetType> {
    match s {
        "archive" => Ok(AssetType::Archive),
        "deb" => Ok(AssetType::Deb),
        "binary" => Ok(AssetType::Binary),
        other => bail!("{ctx}: unknown asset type '{other}'"),
    }
}

fn parse_shell_kind(s: &str, ctx: &str) -> Result<ShellKind> {
    match s {
        "bash" => Ok(ShellKind::Bash),
        "zsh" => Ok(ShellKind::Zsh),
        "fish" => Ok(ShellKind::Fish),
        other => bail!("{ctx}: unknown shell '{other}'"),
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
        (Some(_), Some(_)) => bail!("{ctx}: both self_generated and extracted specified"),
        (None, None) => bail!("{ctx}: neither self_generated nor extracted specified"),
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

    let released_version_parse = raw.released_version_parse.map(|r| {
        ReleasedVersionParseDef {
            tag_starts_with: r.tag_starts_with,
            try_in_body:     r.try_in_body,
        }
    });

    Ok(AppEntry {
        id: raw.id,
        category_id: raw.category_id,
        description: raw.description,
        url: raw.url,
        binaries,
        assets,
        shell_completions,
        man_pages,
        released_version_parse,
    })
}

// ===== Semantic validation (mirrors Registry::collect_rule_errors) =====

fn validate_semantics(apps: &[AppEntry], categories: &[CategoryEntry]) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();

    let category_ids: HashSet<&str> = categories.iter().map(|c| c.id.as_str()).collect();

    let mut global_binary_names: HashMap<String, String> = HashMap::new();
    let mut global_sc_gen: HashMap<(String, String), String> = HashMap::new();
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
            let shell_str = format!("{:?}", sc.shell);
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
                             binary_id={binary_id} shell={shell_str}"
                        ));
                    }
                    let bin_name = app
                        .binaries
                        .iter()
                        .find(|b| b.id == *binary_id)
                        .map(|b| b.name.clone());
                    if let Some(name) = bin_name {
                        let gkey = (name.clone(), shell_str.clone());
                        match global_sc_gen.entry(gkey) {
                            std::collections::hash_map::Entry::Vacant(e) => {
                                e.insert(app_id.clone());
                            }
                            std::collections::hash_map::Entry::Occupied(e) => {
                                let other = e.get();
                                if other != app_id {
                                    errors.push(format!(
                                        "{app_id}: self_generated completion for '{name}' \
                                         {shell_str} conflicts with app '{other}'"
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

// ===== Build entry point =====

fn collect_app_paths(registry_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for entry in std::fs::read_dir(registry_dir)
        .with_context(|| format!("reading {}", registry_dir.display()))?
    {
        let entry = entry?;
        let ft = entry.file_type()?;
        if !ft.is_dir() {
            continue;
        }
        let dir_name = entry.file_name();
        let dir_str = dir_name.to_string_lossy();
        if dir_str.len() != 1 || !dir_str.chars().all(|c| c.is_ascii_lowercase()) {
            continue;
        }
        for file_entry in std::fs::read_dir(entry.path())? {
            let file_entry = file_entry?;
            let name = file_entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.ends_with(".jsonc") {
                paths.push(file_entry.path());
            }
        }
    }
    Ok(paths)
}

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=src/registry/");

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let registry_dir = manifest_dir.join("src/registry");

    // Load and compile JSON schemas
    let app_schema_path = registry_dir.join("app.schema.json");
    let app_schema: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&app_schema_path)
            .with_context(|| format!("reading {}", app_schema_path.display()))?,
    )
    .context("parsing app.schema.json")?;
    let app_validator = jsonschema::validator_for(&app_schema)
        .map_err(|e| anyhow!("invalid app.schema.json: {e}"))?;

    let cat_schema_path = registry_dir.join("categories.schema.json");
    let cat_schema: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&cat_schema_path)
            .with_context(|| format!("reading {}", cat_schema_path.display()))?,
    )
    .context("parsing categories.schema.json")?;
    let cat_validator = jsonschema::validator_for(&cat_schema)
        .map_err(|e| anyhow!("invalid categories.schema.json: {e}"))?;

    let mut all_errors: Vec<String> = Vec::new();

    // Validate and load categories
    let cat_path = registry_dir.join("categories.jsonc");
    let cat_data =
        std::fs::read(&cat_path).with_context(|| format!("reading {}", cat_path.display()))?;
    let cat_value: serde_json::Value = match from_jsonc_slice(&cat_data, "categories.jsonc") {
        Ok(v) => v,
        Err(e) => {
            all_errors.push(format!("categories.jsonc: invalid JSON: {e}"));
            serde_json::Value::Null
        }
    };
    for error in cat_validator.iter_errors(&cat_value) {
        all_errors.push(format!("categories.jsonc: {error}"));
    }
    let categories: Vec<CategoryEntry> = if all_errors.is_empty() {
        let raw: RawCategories = serde_json::from_value(cat_value).context("categories.jsonc")?;
        raw.categories
    } else {
        Vec::new()
    };

    // Collect, validate, and convert all app files
    let mut app_file_paths = collect_app_paths(&registry_dir)?;
    app_file_paths.sort();

    let mut apps: Vec<AppEntry> = Vec::new();
    for path in &app_file_paths {
        let rel = path
            .strip_prefix(&manifest_dir)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");

        let data = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;

        let value: serde_json::Value = match from_jsonc_slice(&data, &rel) {
            Ok(v) => v,
            Err(e) => {
                all_errors.push(format!("{rel}: invalid JSON: {e}"));
                continue;
            }
        };

        for error in app_validator.iter_errors(&value) {
            all_errors.push(format!("{rel}: {error}"));
        }

        let raw: RawApp = match serde_json::from_value(value) {
            Ok(r) => r,
            Err(e) => {
                all_errors.push(format!("{rel}: deserialize error: {e}"));
                continue;
            }
        };

        match convert_app(raw, &rel) {
            Ok(app) => apps.push(app),
            Err(e) => all_errors.push(format!("{rel}: {e}")),
        }
    }

    // Semantic validation
    all_errors.extend(validate_semantics(&apps, &categories));

    if !all_errors.is_empty() {
        for e in &all_errors {
            eprintln!("cargo:error={e}");
        }
        std::process::exit(1);
    }

    // Serialize to postcard and write to OUT_DIR
    let payload = (&categories, &apps);
    let bytes = postcard::to_allocvec(&payload).context("serializing registry to postcard")?;
    std::fs::write(out_dir.join("registry.bin"), &bytes).context("writing registry.bin")?;

    Ok(())
}
