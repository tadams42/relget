use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use json_comments::StripComments;
use serde::Deserialize;
use registry_core::{
    AppAssetDef, AppBinaryDef, AppEntry, AssetType, CategoryEntry,
    CompletionSource, ManPageDef, ReleasedVersionParseDef, ShellCompletionDef, ShellKind,
};

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
    all_errors.extend(registry_core::validate(&apps, &categories));

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
