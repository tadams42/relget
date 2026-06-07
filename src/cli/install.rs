use std::path::PathBuf;

use anyhow::Result;
use clap::Args;

use crate::installer::install_apps;

use super::helpers::{
    DEFAULT_PREFIX, get_codeberg_token, get_github_token, get_gitlab_token, select_apps,
};

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
    log::info!("Installing into: {:?}", args.prefix);
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
