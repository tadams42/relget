use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use clap::Args;

use crate::apps::{App, create_app};

use super::helpers::{DEFAULT_PREFIX, select_apps};

#[derive(Args)]
pub struct UninstallArgs {
    /// Install prefix (e.g. /usr/local or ~/.local)
    #[arg(short = 'p', long, default_value = DEFAULT_PREFIX)]
    pub prefix: PathBuf,

    /// App(s) to uninstall; comma-separated.
    #[arg(
        short = 'a',
        long = "apps",
        value_name = "NAME[,NAME...]",
        value_delimiter = ',',
        conflicts_with_all = ["configured_set"]
    )]
    pub apps: Vec<String>,

    #[arg(
        long,
        value_name = "SET_NAME",
        conflicts_with_all = ["apps"],
        long_help = "Load a named app set from the [sets] table in ~/.config/relget.toml"
    )]
    pub configured_set: Option<String>,
}

pub fn uninstall_command(args: &UninstallArgs) -> Result<()> {
    let selected = select_apps(&args.apps, args.configured_set.as_deref())?;
    let validated = select_apps(&selected, None)?;

    let removed = uninstall_apps(&args.prefix, &validated)?;
    if removed.is_empty() {
        println!("No files removed.");
    } else {
        println!("Removed files:");
        for path in removed {
            println!("- {}", path.display());
        }
    }
    Ok(())
}

pub(super) fn uninstall_apps(prefix: &Path, selected: &[String]) -> Result<Vec<PathBuf>> {
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
