use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use clap::Args;

use crate::apps::{App, create_app};
use crate::clients::RateLimitError;
use crate::types::{AppAssets, AppBinary, Completion, ManPage};

use super::helpers::{
    DEFAULT_PREFIX, get_codeberg_token, get_github_token, get_gitlab_token, select_apps,
};

const BIN_MODE: u32 = 0o755;
const DOC_MODE: u32 = 0o644;

#[derive(Args)]
pub struct InstallArgs {
    /// Install prefix (e.g. /usr/local or ~/.local)
    #[arg(short = 'p', long, default_value = DEFAULT_PREFIX)]
    pub prefix: PathBuf,

    /// App(s) to install; comma-separated.
    #[arg(
        short = 'a',
        long = "apps",
        value_name = "NAME[,NAME...]",
        value_delimiter = ',',
        conflicts_with_all = ["configured_set"]
    )]
    pub apps: Vec<String>,

    /// Load a named app set from the [sets] table in ~/.config/relget.toml
    #[arg(long, value_name = "SET_NAME", conflicts_with_all = ["apps"])]
    pub configured_set: Option<String>,
}

pub fn install_apps_command(args: &InstallArgs, offline: bool) -> Result<()> {
    let selected = select_apps(&args.apps, args.configured_set.as_deref())?;
    log::info!("prefix={:?} msg=Installing", args.prefix);
    let (gh_token, cb_token, gl_token) = if offline {
        (None, None, None)
    } else {
        (get_github_token()?, get_codeberg_token()?, get_gitlab_token()?)
    };
    let installed = install_apps(&args.prefix, &selected, gh_token, cb_token, gl_token, offline)?;
    if !installed.is_empty() {
        println!("Installed files:");
        for path in installed {
            println!("- {}", path.display());
        }
    }

    Ok(())
}

/// Create each app from `selected` and call it's installer
/// - installer might need to download the app, so it may need `gh_token` and/or `cb_token`
/// - if `offline` is true, installer will not try to download anything but will work with cached
///   data only
pub(super) fn install_apps(
    prefix: &Path, selected: &[String], gh_token: Option<String>, cb_token: Option<String>,
    gl_token: Option<String>, offline: bool,
) -> Result<Vec<PathBuf>> {
    let mut installed = Vec::new();
    for app_id in selected {
        let app = create_app(app_id, gh_token.clone(), cb_token.clone(), gl_token.clone(), offline)
            .ok_or_else(|| anyhow!("Unknown app '{}'", app_id))?;
        match install_app(app.as_ref(), prefix) {
            Ok(paths) => installed.extend(paths),
            Err(e) => {
                if e.chain().any(|cause| cause.is::<RateLimitError>()) {
                    log::warn!("app={} msg=Skipping (rate limit): {}", app_id, e.root_cause());
                } else if offline {
                    log::warn!("app={} msg=Skipping (offline, no cached data): {:#}", app_id, e);
                } else {
                    log::error!("app={} msg=Install failed: {:#}", app_id, e);
                }
            }
        }
    }
    Ok(installed)
}

fn install_app(app: &dyn App, prefix: &Path) -> Result<Vec<PathBuf>> {
    if !app.needs_install(prefix)? {
        log::info!("app={} msg=Already at latest version", app.exe_name());
        return Ok(vec![]);
    }
    let assets = app.download()?;
    let installed = install_assets(prefix, &assets)?;
    log::info!("app={} msg=Installed", app.exe_name());
    Ok(installed)
}

fn install_assets(prefix: &Path, assets: &AppAssets) -> Result<Vec<PathBuf>> {
    let mut installed = Vec::new();

    if let Some(bin) = &assets.binary {
        installed.push(install_binary(prefix, bin)?);
    }

    for bin in &assets.other_bins {
        installed.push(install_binary(prefix, bin)?);
    }

    for man in &assets.man_pages {
        installed.push(install_man_page(prefix, man)?);
    }

    for completion in &assets.completions {
        installed.push(install_completion(prefix, completion)?);
    }

    Ok(installed)
}

fn install_binary(prefix: &Path, bin: &AppBinary) -> Result<PathBuf> {
    let dest = bin.install_path(prefix);
    ensure_parent(&dest)?;
    // Write to a temp file then atomically rename into place, matching the standard approach used
    // by dpkg, rpm, and Homebrew. Direct in-place write (O_TRUNC) returns ETXTBSY if any process
    // holds the binary exec-mapped — even a short-lived one. For example, `boring` spawns 5
    // child processes on startup; if they are still alive when we write, the install fails.
    // rename() replaces the directory entry without touching the existing inode, so any running
    // process keeps its mapping on the old inode while the new binary is already in place.
    let tmp = dest.with_extension("relget-tmp");
    fs::write(&tmp, &bin.data).with_context(|| format!("Writing binary to {:?}", dest))?;
    fs::set_permissions(&tmp, fs::Permissions::from_mode(BIN_MODE))?;
    fs::rename(&tmp, &dest).with_context(|| format!("Installing binary to {:?}", dest))?;
    Ok(dest)
}

fn install_man_page(prefix: &Path, man: &ManPage) -> Result<PathBuf> {
    let dest = man.install_path(prefix);
    ensure_parent(&dest)?;
    fs::write(&dest, &man.data).with_context(|| format!("Writing man page to {:?}", dest))?;
    fs::set_permissions(&dest, fs::Permissions::from_mode(DOC_MODE))?;
    Ok(dest)
}

fn install_completion(prefix: &Path, comp: &Completion) -> Result<PathBuf> {
    let dest = comp.install_path(prefix);
    ensure_parent(&dest)?;
    fs::write(&dest, &comp.data).with_context(|| format!("Writing completion to {:?}", dest))?;
    fs::set_permissions(&dest, fs::Permissions::from_mode(DOC_MODE))?;
    Ok(dest)
}

fn ensure_parent(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}
