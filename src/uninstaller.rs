use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};

use crate::apps::{App, create_app};

pub fn uninstall_apps(prefix: &Path, selected: &[String]) -> Result<Vec<PathBuf>> {
    let mut removed = Vec::new();
    for app_id in selected {
        let app = create_app(app_id, None, None, None, false)
            .ok_or_else(|| anyhow!("Unknown app '{}'", app_id))?;
        removed.extend(uninstall_app(&*app, prefix));
    }
    Ok(removed)
}

fn uninstall_app(app: &dyn App, prefix: &Path) -> Vec<PathBuf> {
    let assets = app.assets();
    let mut removed = Vec::new();

    if let Some(bin) = &assets.binary {
        try_remove(bin.install_path(prefix), &mut removed);
    }
    for bin in &assets.other_bins {
        try_remove(bin.install_path(prefix), &mut removed);
    }
    for man in &assets.man_pages {
        try_remove(man.install_path(prefix), &mut removed);
    }
    for comp in &assets.completions {
        try_remove(comp.install_path(prefix), &mut removed);
    }

    removed
}

fn try_remove(path: PathBuf, removed: &mut Vec<PathBuf>) {
    if std::fs::remove_file(&path).is_ok() {
        removed.push(path);
    }
}
