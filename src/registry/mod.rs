use std::collections::HashSet;

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
    pub categories: Vec<RegistryCategory>,
    pub apps:       Vec<RegistryApp>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegistryCategory {
    pub id:          String,
    pub title:       String,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RegistryApp {
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
    categories: Vec<RegistryCategory>,
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
    self_generated: Option<RawSelfGeneratedDef>,
    extracted: Option<RawExtractedDef>,
    ctx: &str,
) -> Result<CompletionSource> {
    match (self_generated, extracted) {
        (Some(sg), None) => Ok(CompletionSource::SelfGenerated {
            binary_id: sg.binary_id,
            command:   sg.command,
        }),
        (None, Some(ex)) => Ok(CompletionSource::Extracted {
            asset_id: ex.asset_id,
            path:     ex.path,
        }),
        (Some(_), Some(_)) => bail!("{}: both self_generated and extracted specified", ctx),
        (None, None) => bail!("{}: neither self_generated nor extracted specified", ctx),
    }
}

fn convert_app(raw: RawApp, path: &str) -> Result<RegistryApp> {
    let binaries = raw
        .binaries
        .into_iter()
        .map(|b| AppBinaryDef {
            id:              b.id,
            name:            b.name,
            version_cmdline: b.version_cmdline,
            is_main:         b.is_main,
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

    Ok(RegistryApp {
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
            let raw: RawApp = serde_json::from_slice(&file.data)
                .with_context(|| format!("parsing {}", path))?;
            apps.push(convert_app(raw, path).with_context(|| format!("converting {}", path))?);
        }

        Ok(Registry {
            categories: raw_cats.categories,
            apps,
        })
    }

    pub fn validate(&self) -> Result<()> {
        let mut errors: Vec<String> = Vec::new();

        // Load and compile schemas
        let app_schema_raw = RegistryFiles::get("app.schema.json")
            .ok_or_else(|| anyhow!("app.schema.json not embedded"))?;
        let cat_schema_raw = RegistryFiles::get("categories.schema.json")
            .ok_or_else(|| anyhow!("categories.schema.json not embedded"))?;

        let app_schema: serde_json::Value = serde_json::from_slice(&app_schema_raw.data)
            .context("parsing app.schema.json")?;
        let cat_schema: serde_json::Value = serde_json::from_slice(&cat_schema_raw.data)
            .context("parsing categories.schema.json")?;

        let app_validator = jsonschema::validator_for(&app_schema)
            .map_err(|e| anyhow!("failed to compile app.schema.json: {}", e))?;
        let cat_validator = jsonschema::validator_for(&cat_schema)
            .map_err(|e| anyhow!("failed to compile categories.schema.json: {}", e))?;

        // Validate categories.json
        let cat_file = RegistryFiles::get("categories.json")
            .ok_or_else(|| anyhow!("categories.json not embedded"))?;
        let cat_value: serde_json::Value =
            serde_json::from_slice(&cat_file.data).context("parsing categories.json")?;
        for error in cat_validator.iter_errors(&cat_value) {
            errors.push(format!("categories.json: {error}"));
        }

        // Build lookup maps from loaded registry
        let category_ids: HashSet<&str> = self.categories.iter().map(|c| c.id.as_str()).collect();
        let apps_by_id: std::collections::HashMap<&str, &RegistryApp> =
            self.apps.iter().map(|a| (a.id.as_str(), a)).collect();

        // Validate each app JSON file
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

            // JSON Schema structural validation
            for error in app_validator.iter_errors(&value) {
                errors.push(format!("{path}: {error}"));
            }

            // Rust-level cross-reference validation
            let app_id = value.get("id").and_then(|v| v.as_str()).unwrap_or(path);
            if let Some(app) = apps_by_id.get(app_id) {
                if !category_ids.contains(app.category_id.as_str()) {
                    errors.push(format!(
                        "{path}: unknown category_id '{}'",
                        app.category_id
                    ));
                }

                let binary_ids: HashSet<u32> = app.binaries.iter().map(|b| b.id).collect();
                let asset_ids: HashSet<u32> = app.assets.iter().map(|a| a.id).collect();

                if binary_ids.len() != app.binaries.len() {
                    errors.push(format!("{path}: duplicate binary ids"));
                }
                if asset_ids.len() != app.assets.len() {
                    errors.push(format!("{path}: duplicate asset ids"));
                }

                let main_count = app.binaries.iter().filter(|b| b.is_main).count();
                if main_count != 1 {
                    errors.push(format!(
                        "{path}: expected exactly 1 binary with is_main=true, found {main_count}"
                    ));
                }

                for sc in &app.shell_completions {
                    match &sc.source {
                        CompletionSource::SelfGenerated { binary_id, .. } => {
                            if !binary_ids.contains(binary_id) {
                                errors.push(format!(
                                    "{path}: shell_completion references unknown binary_id {binary_id}"
                                ));
                            }
                        }
                        CompletionSource::Extracted { asset_id, .. } => {
                            if !asset_ids.contains(asset_id) {
                                errors.push(format!(
                                    "{path}: shell_completion references unknown asset_id {asset_id}"
                                ));
                            }
                        }
                    }
                }

                for mp in &app.man_pages {
                    match &mp.source {
                        CompletionSource::SelfGenerated { binary_id, .. } => {
                            if !binary_ids.contains(binary_id) {
                                errors.push(format!(
                                    "{path}: man_page references unknown binary_id {binary_id}"
                                ));
                            }
                        }
                        CompletionSource::Extracted { asset_id, .. } => {
                            if !asset_ids.contains(asset_id) {
                                errors.push(format!(
                                    "{path}: man_page references unknown asset_id {asset_id}"
                                ));
                            }
                        }
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            for error in &errors {
                eprintln!("{error}");
            }
            bail!("registry validation failed with {} error(s)", errors.len())
        }
    }
}
