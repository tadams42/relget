use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};

use super::helpers;
use crate::{App, RateLimitError, Registry};

pub(super) fn install(
    prefix_path: &Path, apps: &[String], configured_set: Option<&str>, offline: bool,
) -> Result<()> {
    log::info!("prefix={:?} msg=Installing", prefix_path);

    let selected = helpers::select_apps(apps, configured_set)?;
    helpers::check_install_conflicts(prefix_path, &selected, Registry::entries())?;
    let installed = install_apps(prefix_path, &selected, offline)?;

    if !installed.is_empty() {
        println!("Installed files:");
        for path in installed {
            println!("- {}", path.display());
        }
    }

    Ok(())
}

pub(super) fn install_apps(
    prefix_path: &Path, selected: &[String], offline: bool,
) -> Result<Vec<PathBuf>> {
    let (gh_token, cb_token, gl_token) = if offline {
        (None, None, None)
    } else {
        (
            helpers::get_github_token()?,
            helpers::get_codeberg_token()?,
            helpers::get_gitlab_token()?,
        )
    };
    let mut installed = Vec::new();
    for app_id in selected {
        let app =
            App::from_id(app_id, gh_token.clone(), cb_token.clone(), gl_token.clone(), offline)
                .ok_or_else(|| anyhow!("Unknown app '{}'", app_id))?;
        match app.install(prefix_path) {
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
