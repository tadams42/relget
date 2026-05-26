use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow};

use crate::apps::create_app;
use crate::clients::RateLimitError;
use crate::types::{AppBinary, Completion, DownloadedAssets, ManPage, Shell};

const BIN_MODE: u32 = 0o755;
const DOC_MODE: u32 = 0o644;

pub fn install_assets(prefix: &Path, assets: &DownloadedAssets) -> Result<Vec<PathBuf>> {
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
    fs::write(&dest, &bin.data).with_context(|| format!("Writing binary to {:?}", dest))?;
    fs::set_permissions(&dest, fs::Permissions::from_mode(BIN_MODE))?;
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

/// Write binary data to a temp file with executable permissions and run a
/// closure that receives the temp-file path. Temp dir is cleaned up on return.
pub fn with_temp_exe<F, T>(exe_name: &str, data: &[u8], f: F) -> Result<T>
where
    F: FnOnce(&Path) -> Result<T>,
{
    let tmp = tempfile::tempdir()?;
    let exe_path = tmp.path().join(exe_name);
    fs::write(&exe_path, data)?;
    fs::set_permissions(&exe_path, fs::Permissions::from_mode(BIN_MODE))?;
    f(&exe_path)
}

/// Run `exe_path args...` and return stdout bytes.
pub fn run_cmd(exe_path: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let out = Command::new(exe_path)
        .args(args)
        .output()
        .with_context(|| format!("Running {:?} {:?}", exe_path, args))?;
    Ok(out.stdout)
}

/// Generate zsh + bash + fish completions from a single binary with a
/// uniform `[prefix_args..., shell_name]` invocation pattern.
fn gen_completions_with_shell_arg(
    exe_name: &str, data: &[u8], prefix_args: &[&str],
) -> Result<Vec<Completion>> {
    with_temp_exe(exe_name, data, |exe| {
        let mut completions = Vec::new();
        let shells = [
            (Shell::Zsh, "zsh"),
            (Shell::Bash, "bash"),
            (Shell::Fish, "fish"),
        ];
        for (shell, shell_name) in &shells {
            let mut args: Vec<&str> = prefix_args.to_vec();
            args.push(shell_name);
            let stdout = run_cmd(exe, &args)?;
            completions.push(Completion {
                shell:    *shell,
                app_name: exe_name.to_string(),
                data:     stdout,
            });
        }
        Ok(completions)
    })
}

/// `[cmd, subcommand, shell]` pattern (e.g. "starship completions zsh").
pub fn gen_completions_subcommand(
    exe_name: &str, data: &[u8], subcommand: &str,
) -> Result<Vec<Completion>> {
    gen_completions_with_shell_arg(exe_name, data, &[subcommand])
}

/// `[cmd, subcommand, --shell, shell]` pattern (e.g. "atuin gen-completions --shell zsh").
pub fn gen_completions_shell_flag(
    exe_name: &str, data: &[u8], subcommand: &str, flag: &str,
) -> Result<Vec<Completion>> {
    with_temp_exe(exe_name, data, |exe| {
        let mut completions = Vec::new();
        for (shell, shell_name) in &[
            (Shell::Zsh, "zsh"),
            (Shell::Bash, "bash"),
            (Shell::Fish, "fish"),
        ] {
            let stdout = run_cmd(exe, &[subcommand, flag, shell_name])?;
            completions.push(Completion {
                shell:    *shell,
                app_name: exe_name.to_string(),
                data:     stdout,
            });
        }
        Ok(completions)
    })
}

/// Create each app from `selected` and call it's installer
/// - installer might need to download the app, so it may need `gh_token` and/or `cb_token`
/// - if `offline` is true, installer will not try to download anything but will work with cached
///   data only
pub fn install_apps(
    prefix: &Path, selected: &[String], gh_token: Option<String>, cb_token: Option<String>,
    offline: bool,
) -> Result<Vec<PathBuf>> {
    let mut installed = Vec::new();
    for app_id in selected {
        let app = create_app(app_id, gh_token.clone(), cb_token.clone(), offline)
            .ok_or_else(|| anyhow!("Unknown app '{}'", app_id))?;
        match app.install(prefix) {
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
