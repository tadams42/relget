use std::collections::{HashMap, HashSet};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result, bail};

use super::App;
use super::app_assets::BIN_MODE;
use super::app_trait::run_cmd;
use crate::registry::{
    AppAssetDef, AppBinaryDef, AppEntry, AssetType, CompletionSource, ShellKind,
};
use crate::{
    AppAssets, AppBinary, AppVersion, ArchiveExtractor, CodebergClient, Completion, GithubClient,
    GitlabClient, ManPage, RelgetClient, Shell,
};

pub struct GenericApp {
    entry:  AppEntry,
    client: Arc<dyn RelgetClient>,
}

impl GenericApp {
    pub fn new(entry: AppEntry, client: Arc<dyn RelgetClient>) -> Self { Self { entry, client } }

    pub fn client_for(
        entry: &AppEntry, gh_token: Option<String>, cb_token: Option<String>,
        gl_token: Option<String>, offline: bool,
    ) -> Arc<dyn RelgetClient> {
        let url = &entry.url;
        if url.contains("codeberg.org") {
            Arc::new(CodebergClient::new(cb_token, offline))
        } else if url.contains("gitlab.com") {
            Arc::new(GitlabClient::new(gl_token, offline))
        } else {
            Arc::new(GithubClient::new(gh_token, offline))
        }
    }

    fn owner_repo(url: &str) -> (&str, &str) {
        let without_scheme = url.split_once("://").map(|(_, rest)| rest).unwrap_or(url);
        let mut parts = without_scheme.splitn(3, '/');
        let _host = parts.next().unwrap_or("");
        let owner = parts.next().unwrap_or("");
        let repo = parts.next().unwrap_or("");
        (owner, repo)
    }

    fn matches_asset(def: &AppAssetDef, name: &str) -> bool {
        def.starts_with
            .as_deref()
            .is_none_or(|s| name.starts_with(s))
            && def.contains.as_deref().is_none_or(|s| name.contains(s))
            && def
                .not_contains
                .as_deref()
                .is_none_or(|s| !name.contains(s))
            && def.ends_with.as_deref().is_none_or(|s| name.ends_with(s))
            && def.equals.as_deref().is_none_or(|s| name == s)
    }

    fn shell_kind_to_shell(sk: &ShellKind) -> Shell {
        match sk {
            ShellKind::Bash => Shell::Bash,
            ShellKind::Zsh => Shell::Zsh,
            ShellKind::Fish => Shell::Fish,
        }
    }

    fn app_name_from_path(path: &str, shell: &ShellKind) -> String {
        let base = Path::new(path)
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_owned());
        match shell {
            ShellKind::Zsh => {
                if let Some(s) = base.strip_prefix('_') {
                    s.to_owned()
                } else if let Some(s) = base.strip_suffix(".zsh") {
                    s.to_owned()
                } else {
                    base
                }
            }
            ShellKind::Bash => {
                base.strip_suffix(".bash")
                    .map(str::to_owned)
                    .unwrap_or(base)
            }
            ShellKind::Fish => {
                base.strip_suffix(".fish")
                    .map(str::to_owned)
                    .unwrap_or(base)
            }
        }
    }

    fn man_filename_from_path(path: &str) -> String {
        Path::new(path)
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_owned())
    }

    fn binary_name_by_id(&self, id: u32) -> &str {
        self.entry
            .binaries
            .iter()
            .find(|b| b.id == id)
            .map(|b| b.name.as_str())
            .expect("registry validation ensures binary_id exists")
    }

    fn extract_binary_data(
        binary_def: &AppBinaryDef, assets: &[AppAssetDef],
        downloaded: &HashMap<u32, (String, Vec<u8>)>, content_asset_ids: &HashSet<u32>,
    ) -> Result<Vec<u8>> {
        // Try archive/deb assets in id order
        for asset_def in assets
            .iter()
            .filter(|a| !matches!(a.asset_type, AssetType::Binary))
        {
            if let Some((name, data)) = downloaded.get(&asset_def.id) {
                let extractor = ArchiveExtractor::new(name.as_str(), data.clone());

                if let Ok(extracted) = extractor.extract_by_filename(&binary_def.name) {
                    return Ok(extracted);
                }

                // Single-member archive (handles single-file .gz like dasel)
                if let Ok(members) = extractor.members() {
                    if members.len() == 1 {
                        if let Ok(extracted) = extractor.extract(&members[0]) {
                            return Ok(extracted);
                        }
                    }
                }
            }
        }

        // Fallback: Binary-type asset not referenced by any Extracted source
        for asset_def in assets
            .iter()
            .filter(|a| matches!(a.asset_type, AssetType::Binary))
        {
            if content_asset_ids.contains(&asset_def.id) {
                continue;
            }
            if let Some((_, data)) = downloaded.get(&asset_def.id) {
                return Ok(data.clone());
            }
        }

        bail!(
            "Cannot extract binary '{}': not found in any downloaded asset",
            binary_def.name
        )
    }

    fn extract_from_asset(
        asset_def: &AppAssetDef, asset_name: &str, asset_data: &[u8], path: &str,
    ) -> Result<Vec<u8>> {
        if matches!(asset_def.asset_type, AssetType::Binary) {
            Ok(asset_data.to_vec())
        } else {
            let extractor = ArchiveExtractor::new(asset_name, asset_data.to_vec());
            let filename = Path::new(path)
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or(path);
            extractor.extract_by_filename(filename)
        }
    }

    /// Extract from archive, decompressing a `.gz` inner file if the installed path
    /// doesn't end in `.gz` but the archive only has the compressed form.
    fn extract_man_page(
        asset_def: &AppAssetDef, asset_name: &str, asset_data: &[u8], path: &str,
    ) -> Result<Vec<u8>> {
        if matches!(asset_def.asset_type, AssetType::Binary) {
            return Ok(asset_data.to_vec());
        }
        let extractor = ArchiveExtractor::new(asset_name, asset_data.to_vec());
        let filename = Path::new(path)
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or(path);
        // Direct extraction first
        if let Ok(data) = extractor.extract_by_filename(filename) {
            return Ok(data);
        }
        // If not found and path doesn't end in .gz, try the compressed form
        if !filename.ends_with(".gz") {
            let gz_name = format!("{filename}.gz");
            if let Ok(compressed) = extractor.extract_by_filename(&gz_name) {
                let decompressor = ArchiveExtractor::new("inner.gz", compressed);
                return decompressor.extract("inner");
            }
        }
        bail!("Cannot extract man page '{}' from asset '{}'", filename, asset_name)
    }
}

impl App for GenericApp {
    fn exe_name(&self) -> &str { self.entry.main_exe_name() }

    fn cli_version_arg(&self) -> &str {
        self.entry
            .binaries
            .iter()
            .find(|b| b.is_main)
            .map(|b| b.version_cmdline.as_str())
            .unwrap_or("--version")
    }

    fn released_version(&self) -> Result<AppVersion> {
        let (owner, repo) = Self::owner_repo(&self.entry.url);
        self.client.latest_release(owner, repo)?.version()
    }

    fn assets(&self) -> AppAssets {
        let main_bin = self.entry.binaries.iter().find(|b| b.is_main).unwrap();
        let binary = Some(AppBinary::new(&main_bin.name));
        let other_bins: Vec<AppBinary> = self
            .entry
            .binaries
            .iter()
            .filter(|b| !b.is_main)
            .map(|b| AppBinary::new(&b.name))
            .collect();

        let completions: Vec<Completion> = self
            .entry
            .shell_completions
            .iter()
            .map(|sc| {
                let shell = Self::shell_kind_to_shell(&sc.shell);
                let app_name = match &sc.source {
                    CompletionSource::SelfGenerated { binary_id, .. } => {
                        self.binary_name_by_id(*binary_id).to_owned()
                    }
                    CompletionSource::Extracted { path, .. } => {
                        Self::app_name_from_path(path, &sc.shell)
                    }
                };
                Completion::new(shell, app_name)
            })
            .collect();

        let man_pages: Vec<ManPage> = self
            .entry
            .man_pages
            .iter()
            .filter(|mp| {
                // Batch generators ({{ tmp-dir }}) produce a dynamic set of files at runtime;
                // the extracted entries alongside them serve as the static list for assets()/uninstaller
                !matches!(&mp.source, CompletionSource::SelfGenerated { command, .. } if command.contains("{{ tmp-dir }}"))
            })
            .map(|mp| {
                let file_name = match &mp.source {
                    CompletionSource::SelfGenerated { binary_id, .. } => {
                        format!("{}.{}", self.binary_name_by_id(*binary_id), mp.section)
                    }
                    CompletionSource::Extracted { path, .. } => Self::man_filename_from_path(path),
                };
                ManPage::new(mp.section, file_name)
            })
            .collect();

        AppAssets {
            binary,
            other_bins,
            completions,
            man_pages,
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let (owner, repo) = Self::owner_repo(&self.entry.url);
        let release = self.client.latest_release(owner, repo)?;

        // Download all defined assets
        let mut downloaded: HashMap<u32, (String, Vec<u8>)> = HashMap::new();
        for asset_def in &self.entry.assets {
            // "tarball" is a sentinel that bypasses find_asset and fetches the source tarball
            let is_tarball = asset_def.equals.as_deref() == Some("tarball");
            let name = if is_tarball {
                "tarball".to_owned()
            } else {
                release.find_asset(|a| Self::matches_asset(asset_def, a))?
            };
            let cached = self.client.download_asset(owner, repo, &name)?;
            // ArchiveExtractor needs a recognizable extension; tarball is always .tar.gz
            let archive_name = if is_tarball {
                format!("{}.tar.gz", cached.name)
            } else {
                name
            };
            downloaded.insert(asset_def.id, (archive_name, cached.data));
        }

        // Detect batch man page generator — generates multiple files to a tmpdir at runtime
        let has_batch_man_gen = self.entry.man_pages.iter().any(|mp| {
            matches!(&mp.source, CompletionSource::SelfGenerated { command, .. } if command.contains("{{ tmp-dir }}"))
        });

        // Collect asset IDs used exclusively for content (completions/man pages)
        let content_asset_ids: HashSet<u32> = self
            .entry
            .shell_completions
            .iter()
            .filter_map(|sc| {
                match &sc.source {
                    CompletionSource::Extracted { asset_id, .. } => Some(*asset_id),
                    _ => None,
                }
            })
            .chain(self.entry.man_pages.iter().filter_map(|mp| {
                // When a batch generator is present, extracted entries are metadata-only
                // (never downloaded), so exclude their asset_ids from this protection set
                if has_batch_man_gen {
                    return None;
                }
                match &mp.source {
                    CompletionSource::Extracted { asset_id, .. } => Some(*asset_id),
                    _ => None,
                }
            }))
            .collect();

        // Extract all binaries
        let mut binary_data: HashMap<String, Vec<u8>> = HashMap::new();
        for bin in &self.entry.binaries {
            let data = Self::extract_binary_data(
                bin,
                &self.entry.assets,
                &downloaded,
                &content_asset_ids,
            )?;
            binary_data.insert(bin.name.clone(), data);
        }

        // Run self-generated completions and man pages with all binaries in a temp dir
        let mut completions: Vec<Completion> = Vec::new();
        let mut man_pages: Vec<ManPage> = Vec::new();

        let has_self_gen = self
            .entry
            .shell_completions
            .iter()
            .any(|sc| matches!(sc.source, CompletionSource::SelfGenerated { .. }))
            || self
                .entry
                .man_pages
                .iter()
                .any(|mp| matches!(mp.source, CompletionSource::SelfGenerated { .. }));

        if has_self_gen {
            let tmp = tempfile::tempdir()?;
            for (name, data) in &binary_data {
                let path = tmp.path().join(name);
                std::fs::write(&path, data)?;
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(BIN_MODE))?;
            }

            for sc in &self.entry.shell_completions {
                if let CompletionSource::SelfGenerated { binary_id, command } = &sc.source {
                    let bin_name = self.binary_name_by_id(*binary_id);
                    let exe_path = tmp.path().join(bin_name);
                    let args: Vec<&str> = command.split_whitespace().collect();
                    let data = run_cmd(&exe_path, &args)?;
                    completions.push(Completion::new_with_data(
                        Self::shell_kind_to_shell(&sc.shell),
                        bin_name,
                        data,
                    ));
                }
            }

            for mp in &self.entry.man_pages {
                if let CompletionSource::SelfGenerated { binary_id, command } = &mp.source {
                    let bin_name = self.binary_name_by_id(*binary_id);
                    let exe_path = tmp.path().join(bin_name);

                    if command.contains("{{ tmp-dir }}") {
                        // Batch generator: writes multiple files into tmpdir; collect all of them
                        let man_tmp = tempfile::tempdir()?;
                        let dir_str = man_tmp.path().to_str().context("non-UTF8 temp dir")?;
                        let args: Vec<String> = command
                            .split_whitespace()
                            .map(|t| t.replace("{{ tmp-dir }}", dir_str))
                            .collect();
                        let args_refs: Vec<&str> = args.iter().map(String::as_str).collect();
                        run_cmd(&exe_path, &args_refs)?;
                        for dir_entry in std::fs::read_dir(man_tmp.path())? {
                            let dir_entry = dir_entry?;
                            let path = dir_entry.path();
                            if !path.is_file() {
                                continue;
                            }
                            let filename = path
                                .file_name()
                                .and_then(|f| f.to_str())
                                .map(str::to_owned)
                                .ok_or_else(|| {
                                    anyhow::anyhow!("non-UTF8 filename in man page tmpdir")
                                })?;
                            let data = std::fs::read(&path)?;
                            man_pages.push(ManPage::new_with_data(mp.section, filename, data));
                        }
                    } else {
                        let filename = format!("{}.{}", bin_name, mp.section);
                        let args: Vec<&str> = command.split_whitespace().collect();
                        let data = run_cmd(&exe_path, &args)?;
                        man_pages.push(ManPage::new_with_data(mp.section, filename, data));
                    }
                }
            }
        }

        // Extracted completions
        for sc in &self.entry.shell_completions {
            if let CompletionSource::Extracted { asset_id, path } = &sc.source {
                let (asset_name, asset_data) = downloaded
                    .get(asset_id)
                    .expect("registry validates asset_id");
                let asset_def = self
                    .entry
                    .assets
                    .iter()
                    .find(|a| a.id == *asset_id)
                    .expect("registry validates asset_id");
                let data = Self::extract_from_asset(asset_def, asset_name, asset_data, path)?;
                completions.push(Completion::new_with_data(
                    Self::shell_kind_to_shell(&sc.shell),
                    Self::app_name_from_path(path, &sc.shell),
                    data,
                ));
            }
        }

        // Extracted man pages — skipped when a batch generator has already handled them
        if !has_batch_man_gen {
            for mp in &self.entry.man_pages {
                if let CompletionSource::Extracted { asset_id, path } = &mp.source {
                    let (asset_name, asset_data) = downloaded
                        .get(asset_id)
                        .expect("registry validates asset_id");
                    let asset_def = self
                        .entry
                        .assets
                        .iter()
                        .find(|a| a.id == *asset_id)
                        .expect("registry validates asset_id");
                    let data = Self::extract_man_page(asset_def, asset_name, asset_data, path)?;
                    let filename = Self::man_filename_from_path(path);
                    man_pages.push(ManPage::new_with_data(mp.section, filename, data));
                }
            }
        }

        // Assemble result
        let main_def = self.entry.binaries.iter().find(|b| b.is_main).unwrap();
        let main_data = binary_data.remove(&main_def.name).unwrap();
        let binary = Some(AppBinary::new_with_data(&main_def.name, main_data));
        let other_bins: Vec<AppBinary> = self
            .entry
            .binaries
            .iter()
            .filter(|b| !b.is_main)
            .map(|b| {
                let data = binary_data.remove(&b.name).unwrap();
                AppBinary::new_with_data(&b.name, data)
            })
            .collect();

        Ok(AppAssets {
            binary,
            other_bins,
            completions,
            man_pages,
        })
    }
}
