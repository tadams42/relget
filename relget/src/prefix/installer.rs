use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow, bail};

use super::helpers;
use crate::{App, RateLimitError, Registry};

pub(super) fn install(
    prefix_path: &Path, apps: &[String], configured_set: Option<&str>, offline: bool,
) -> Result<()> {
    log::info!("prefix={prefix_path:?} msg=Installing");

    let selected = helpers::select_apps(apps, configured_set)?;
    helpers::check_install_conflicts(prefix_path, &selected, Registry::entries())?;
    let (installed, failed) = install_apps(prefix_path, &selected, offline)?;

    if !installed.is_empty() {
        println!("Installed files:");
        for path in installed {
            println!("- {}", path.display());
        }
    }
    if failed > 0 {
        bail!("{failed} app(s) failed to install");
    }

    Ok(())
}

/// Installs `selected` apps, returning the installed file paths and the number of hard
/// failures. Rate-limit and offline cache misses are deliberate soft skips (logged as
/// warnings) and do not count as failures.
pub(super) fn install_apps(
    prefix_path: &Path, selected: &[String], offline: bool,
) -> Result<(Vec<PathBuf>, usize)> {
    let (gh_token, cb_token, gl_token) = if offline {
        (None, None, None)
    } else {
        (
            crate::Config::github_token()?,
            crate::Config::codeberg_token()?,
            crate::Config::gitlab_token()?,
        )
    };
    let mut installed = Vec::new();
    let mut failed = 0;
    for app_id in selected {
        let app =
            App::from_id(app_id, gh_token.clone(), cb_token.clone(), gl_token.clone(), offline)
                .ok_or_else(|| anyhow!("Unknown app '{app_id}'"))?;
        match app.install(prefix_path) {
            Ok(paths) => installed.extend(paths),
            Err(e) => {
                if e.chain().any(|cause| cause.is::<RateLimitError>()) {
                    log::warn!("app={} msg=Skipping (rate limit): {}", app_id, e.root_cause());
                } else if offline {
                    log::warn!("app={app_id} msg=Skipping (offline, no cached data): {e:#}");
                } else {
                    log::error!("app={app_id} msg=Install failed: {e:#}");
                    failed += 1;
                }
            }
        }
    }
    Ok((installed, failed))
}
