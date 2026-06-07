use std::path::PathBuf;

use anyhow::Result;
use clap::Args;

use crate::uninstaller::uninstall_apps;

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

    /// Load a named app set from the [sets] table in ~/.config/relget.toml
    #[arg(long, value_name = "SET_NAME", conflicts_with_all = ["apps"])]
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
