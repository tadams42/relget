use std::path::PathBuf;

use anyhow::Result;
use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use clap_complete::{Shell, generate};

use crate::apps::minimal_set_identifiers;

use super::sub_commands::{
    install_apps_command, list_apps_ids_command, uninstall_command, update_command,
};

const DEFAULT_PREFIX: &str = "/usr/local";

fn styles() -> Styles {
    Styles::styled()
        .header(
            AnsiColor::Green
                .on_default()
                .effects(Effects::BOLD | Effects::UNDERLINE),
        )
        .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
        .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
        .placeholder(AnsiColor::Cyan.on_default())
        .error(AnsiColor::Red.on_default().effects(Effects::BOLD))
        .valid(AnsiColor::Green.on_default().effects(Effects::BOLD))
        .invalid(AnsiColor::Yellow.on_default().effects(Effects::BOLD))
}

pub fn create_cli() -> Result<Cli> {
    let minimal_set_help = format!(
        "Install a hand-picked minimal set of apps (overrides --apps): {}",
        minimal_set_identifiers().join(", ")
    );
    let cmd = Cli::command()
        .styles(styles())
        .mut_arg("minimal_set", |a| a.help(minimal_set_help));
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
        Some(Commands::Update) => update_command(cli)?,
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
        global = true,
        conflicts_with_all = ["minimal_set", "configured_set"]
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

    /// Install a hand-picked minimal set of apps
    #[arg(long, default_value_t = false, global = true, conflicts_with_all = ["apps", "configured_set"])]
    pub minimal_set: bool,

    /// Load a named app set from the [sets] table in ~/.config/relget.toml
    #[arg(long, value_name = "SET_NAME", global = true, conflicts_with_all = ["apps", "minimal_set"])]
    pub configured_set: Option<String>,

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
    /// Uninstall selected apps from the given prefix
    ///
    /// Removes exactly the files that relget installed: the binary, all secondary binaries,
    /// man pages, and shell completions. The set of files is derived from each app's static
    /// asset descriptor, so every file relget placed is removed — no more, no less.
    ///
    /// Token flags (--gh-token-source, --cb-token-source, --gl-token-source) are accepted but
    /// ignored for this command.
    #[command(verbatim_doc_comment)]
    Uninstall,
    /// Update relget-managed apps in the prefix
    ///
    /// Without selectors: scans `<prefix>/bin/` for executables that match a known app in the
    /// registry and updates each one. When a binary name matches more than one registry entry
    /// (e.g. `qsv` for both `qsv` and `qsv-all`), the first alphabetical match is used and a
    /// warning is printed.
    ///
    /// With --apps / --minimal-set / --configured-set: updates only the specified apps,
    /// regardless of whether they are currently installed.
    ///
    /// Apps already at the latest version are skipped in both cases.
    #[command(verbatim_doc_comment)]
    Update,
}
