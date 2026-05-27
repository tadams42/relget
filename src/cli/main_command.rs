use std::path::PathBuf;

use anyhow::Result;
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use clap_complete::{Shell, generate};

use crate::apps::minimal_set_identifiers;

use super::sub_commands::{
    install_apps_command, list_apps_ids_command, reinstall_apps_command, uninstall_command,
};

const DEFAULT_PREFIX: &str = "/usr/local";

pub fn create_cli() -> Result<Cli> {
    let minimal_set_help = format!(
        "Install a hand-picked minimal set of apps (overrides --apps): {}",
        minimal_set_identifiers().join(", ")
    );
    let cmd = Cli::command().mut_arg("minimal_set", |a| a.help(minimal_set_help));
    let cli = Cli::from_arg_matches(&cmd.get_matches())?;

    Ok(cli)
}

pub fn execute_cli(cli: &Cli) -> Result<()> {
    match cli.command {
        Some(Commands::ListAppsIds) => list_apps_ids_command(),
        Some(Commands::Completions { shell }) => {
            generate(shell, &mut Cli::command(), "relget", &mut std::io::stdout())
        }
        Some(Commands::Uninstall) => uninstall_command(cli)?,
        Some(Commands::Reinstall) => reinstall_apps_command(cli)?,
        None => install_apps_command(cli)?,
    }

    Ok(())
}

#[derive(Parser)]
#[command(name = "relget")]
#[command(version)]
#[command(about = "Installs or updates CLI utilities directly from GitHub releases")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Install prefix (e.g. /usr/local or ~/.local)
    #[arg(short = 'p', long, default_value = DEFAULT_PREFIX, global = true)]
    pub prefix: PathBuf,

    /// App(s) to install; comma-separated.
    #[arg(
        short = 'a',
        long = "apps",
        value_name = "NAME[,NAME...]",
        value_delimiter = ',',
        global = true
    )]
    pub apps: Vec<String>,

    /// GitHub token source: `prompt` to enter interactively, `load` to read from
    /// RELGET_GHB_TOKEN env var or ~/.config/relget.toml (github_token key)
    #[arg(long, default_value = "load", value_parser = ["prompt", "load"], global = true)]
    pub gh_token_source: String,

    /// Codeberg token source: `prompt` to enter interactively, `load` to read from
    /// RELGET_CDB_TOKEN env var or ~/.config/relget.toml (codeberg_token key)
    #[arg(long, default_value = "load", value_parser = ["prompt", "load"], global = true)]
    pub cb_token_source: String,

    /// GitLab token source: `prompt` to enter interactively, `load` to read from
    /// RELGET_GLB_TOKEN env var or ~/.config/relget.toml (gitlab_token key)
    #[arg(long, default_value = "load", value_parser = ["prompt", "load"], global = true)]
    pub gl_token_source: String,

    /// Install a hand-picked minimal set of apps (overrides --apps)
    #[arg(long, default_value_t = false, global = true)]
    pub minimal_set: bool,

    /// Use only cached data; never make network requests
    #[arg(long, default_value_t = false, global = true)]
    pub offline: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Print all supported app identifiers
    ListAppsIds,
    /// Print shell completion script to stdout
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
    /// Uninstall selected apps from the given prefix (best-effort)
    ///
    /// Removes the binary, shell completion files, and man pages that match
    /// the app's executable name under the given prefix. This is a
    /// best-effort operation:
    ///
    ///   - Only files that follow relget's standard installation layout are searched. Files placed
    ///     elsewhere will not be touched.
    ///
    ///   - Apps that install multiple binaries under different names (e.g. `uv` also installs
    ///     `uvx`) will have only the primary binary removed. The secondary binaries and their
    ///     completions are left behind.
    ///
    ///   - Man pages with a separator other than `-` (e.g. `eza_colors.5`) are not matched and will
    ///     not be removed.
    ///
    /// Token flags (--gh-token-source, --cb-token-source, --gl-token-source) are accepted but
    /// ignored for this command.
    #[command(verbatim_doc_comment)]
    Uninstall,
    /// Uninstall then reinstall selected apps
    ///
    /// Equivalent to running `uninstall` followed by the default install.
    /// Useful to force a clean reinstall without manually tracking installed
    /// files. Inherits the same best-effort caveats as `uninstall`.
    Reinstall,
}
