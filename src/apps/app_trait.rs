use crate::types::{AppAssets, Completion, Shell};
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
            completions.push(Completion {
                shell:    *shell,
                app_name: exe_name.to_string(),
                data:     stdout,
            });
        }
        Ok(completions)
    })
}

pub trait App {
    fn exe_name(&self) -> &str;

    fn cli_version_arg(&self) -> &str { DEFAULT_VERSION_ARG }

    fn released_version(&self) -> Result<AppVersion>;

    fn assets(&self) -> AppAssets;

    fn download(&self) -> Result<AppAssets>;

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

    fn needs_install(&self, prefix: &Path) -> Result<bool> {
        let installed = self.installed_version(prefix)?;
        match installed {
            None => Ok(true),
            Some(iv) => Ok(iv != self.released_version()?),
        }
    }
}
