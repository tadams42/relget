use crate::apps::app_assets::{AppAssets, Completion, Shell};
use crate::version::AppVersion;
use anyhow::{Context, Result};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

const DEFAULT_VERSION_ARG: &str = "--version";
const BIN_MODE: u32 = 0o755;

pub(in crate::apps) fn with_temp_exe<F, T>(exe_name: &str, data: &[u8], f: F) -> Result<T>
where
    F: FnOnce(&Path) -> Result<T>,
{
    let tmp = tempfile::tempdir()?;
    let exe_path = tmp.path().join(exe_name);
    fs::write(&exe_path, data)?;
    fs::set_permissions(&exe_path, fs::Permissions::from_mode(BIN_MODE))?;
    f(&exe_path)
}

pub(in crate::apps) fn run_cmd(exe_path: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let out = Command::new(exe_path)
        .args(args)
        .output()
        .with_context(|| format!("Running {:?} {:?}", exe_path, args))?;
    Ok(out.stdout)
}

pub(in crate::apps) fn gen_completions_with_shell_arg(
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
            completions.push(Completion::new_with_data(*shell, exe_name, stdout));
        }
        Ok(completions)
    })
}

pub(in crate::apps) fn gen_completions_subcommand(
    exe_name: &str, data: &[u8], subcommand: &str,
) -> Result<Vec<Completion>> {
    gen_completions_with_shell_arg(exe_name, data, &[subcommand])
}

pub(in crate::apps) fn gen_completions_shell_flag(
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
            completions.push(Completion::new_with_data(*shell, exe_name, stdout));
        }
        Ok(completions)
    })
}

/// Describes a single installable CLI application sourced from a GitHub, GitLab, or Codeberg
/// release.
///
/// Implementors must ensure that [`App::assets`] and [`App::download`] return **exactly the same
/// set of files** — `assets()` drives the uninstaller, so any mismatch leaves orphaned or
/// untracked files on disk.
pub trait App {
    /// Returns the name of the primary installed binary (e.g. `"rg"`, `"bat"`).
    ///
    /// Used to locate the binary under `<prefix>/bin/` when checking [`App::installed_version`].
    fn exe_name(&self) -> &str;

    /// Returns the CLI argument used to print the binary's version string.
    ///
    /// Defaults to `--version`. Override when the app uses a subcommand (`"version"`) or a short
    /// flag (`"-v"`). An incorrect value makes [`App::installed_version`] always return `None`,
    /// causing the app to be reinstalled on every `update` run.
    fn cli_version_arg(&self) -> &str { DEFAULT_VERSION_ARG }

    /// Fetches the latest release version from the upstream API (GitHub, GitLab, or Codeberg).
    ///
    /// # Errors
    /// Returns an error if the API request fails or the response cannot be parsed.
    fn released_version(&self) -> Result<AppVersion>;

    /// Returns a static descriptor of every file this app installs — binaries, man pages, and
    /// shell completions — with empty `data` fields.
    ///
    /// This is the source of truth for the uninstaller. It **must** describe exactly the same
    /// files that [`App::download`] writes to disk; any omission leaves orphaned files, any
    /// addition causes the uninstaller to attempt removing files that were never installed.
    fn assets(&self) -> AppAssets;

    /// Downloads and extracts the latest release, returning populated [`AppAssets`].
    ///
    /// The returned asset set must match [`App::assets`] exactly — same files, same names.
    ///
    /// # Errors
    /// Returns an error if the download, extraction, or any post-processing step (e.g.
    /// generating completions by running the binary) fails.
    fn download(&self) -> Result<AppAssets>;

    /// Returns the version of the currently installed binary, or `None` if not installed.
    ///
    /// Runs `<prefix>/bin/<exe_name> <cli_version_arg>` and scans combined stdout+stderr for a
    /// semver-like version string via [`AppVersion::find_in`]. Returns `None` if the binary is
    /// absent or the version cannot be parsed.
    ///
    /// # Errors
    /// Only propagates hard I/O errors; a binary that exits non-zero is treated as `None`.
    fn installed_version(&self, prefix: &Path) -> Result<Option<AppVersion>> {
        let bin = prefix.join("bin").join(self.exe_name());
        if !bin.exists() {
            return Ok(None);
        }
        let out = std::process::Command::new(&bin)
            .arg(self.cli_version_arg())
            .output();
        match out {
            Err(_) => Ok(None),
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let stderr = String::from_utf8_lossy(&o.stderr);
                let combined = format!("{}{}", stdout, stderr);
                Ok(AppVersion::find_in(&combined))
            }
        }
    }

    /// Returns `true` if the app is not installed or the installed version differs from the
    /// latest released version.
    ///
    /// # Errors
    /// Returns an error if either [`App::installed_version`] or [`App::released_version`] fails.
    fn needs_install(&self, prefix: &Path) -> Result<bool> {
        let installed = self.installed_version(prefix)?;
        match installed {
            None => Ok(true),
            Some(iv) => Ok(iv != self.released_version()?),
        }
    }
}
