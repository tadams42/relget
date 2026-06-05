use std::path::PathBuf;

use anyhow::Result;
use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::{Args, CommandFactory, FromArgMatches, Parser, Subcommand};
use clap_complete::{Shell, generate};

use crate::apps::minimal_set_identifiers;

use super::doctor::doctor_command;
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
    let app_list = minimal_set_identifiers().join(", ");
    let install_help = format!("Install a hand-picked minimal set of apps: {}", app_list);
    let update_help = format!("Update a hand-picked minimal set of apps: {}", app_list);
    let uninstall_help = format!("Uninstall a hand-picked minimal set of apps: {}", app_list);
    let mut cmd = Cli::command()
        .styles(styles())
        .mut_subcommand("install", |sub| {
            sub.mut_arg("minimal_set", |a| a.help(install_help.clone()))
        })
        .mut_subcommand("update", |sub| {
            sub.mut_arg("minimal_set", |a| a.help(update_help.clone()))
        })
        .mut_subcommand("uninstall", |sub| {
            sub.mut_arg("minimal_set", |a| a.help(uninstall_help.clone()))
        });

    let matches = match cmd.clone().try_get_matches() {
        Ok(m) => m,
        Err(e) => {
            match e.kind() {
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion => {
                    e.exit()
                }
                clap::error::ErrorKind::MissingSubcommand => {
                    let _ = cmd.print_help();
                    eprintln!();
                    std::process::exit(2);
                }
                _ => e.exit(),
            }
        }
    };

    Ok(Cli::from_arg_matches(&matches)?)
}

pub fn execute_cli(cli: &Cli) -> Result<()> {
    match &cli.command {
        Commands::Install(args) => install_apps_command(args, cli.offline)?,
        Commands::Update(args) => update_command(args, cli.offline)?,
        Commands::Uninstall(args) => uninstall_command(args)?,
        Commands::Doctor(args) => doctor_command(args, cli.offline)?,
        Commands::ListAppsIds => list_apps_ids_command(),
        Commands::Completions { shell } => {
            generate(*shell, &mut Cli::command(), "relget", &mut std::io::stdout())
        }
    }

    Ok(())
}

#[derive(Parser)]
#[command(name = "relget")]
#[command(version)]
#[command(about = "Installs or updates CLI utilities directly from GitHub releases")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Use only cached data; never make network requests
    #[arg(long, default_value_t = false, global = true)]
    pub offline: bool,
}

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
        conflicts_with_all = ["minimal_set", "configured_set"]
    )]
    pub apps: Vec<String>,

    /// GitHub token source: `prompt` to enter interactively, `load` to read from
    /// RELGET_GHB_TOKEN env var or ~/.config/relget.toml (github_token key)
    #[arg(long, default_value = "load", value_parser = ["prompt", "load"])]
    pub gh_token_source: String,

    /// Codeberg token source: `prompt` to enter interactively, `load` to read from
    /// RELGET_CDB_TOKEN env var or ~/.config/relget.toml (codeberg_token key)
    #[arg(long, default_value = "load", value_parser = ["prompt", "load"])]
    pub cb_token_source: String,

    /// GitLab token source: `prompt` to enter interactively, `load` to read from
    /// RELGET_GLB_TOKEN env var or ~/.config/relget.toml (gitlab_token key)
    #[arg(long, default_value = "load", value_parser = ["prompt", "load"])]
    pub gl_token_source: String,

    /// Install a hand-picked minimal set of apps
    #[arg(long, default_value_t = false, conflicts_with_all = ["apps", "configured_set"])]
    pub minimal_set: bool,

    /// Load a named app set from the [sets] table in ~/.config/relget.toml
    #[arg(long, value_name = "SET_NAME", conflicts_with_all = ["apps", "minimal_set"])]
    pub configured_set: Option<String>,
}

#[derive(Args)]
pub struct UpdateArgs {
    /// Install prefix (e.g. /usr/local or ~/.local)
    #[arg(short = 'p', long, default_value = DEFAULT_PREFIX)]
    pub prefix: PathBuf,

    /// App(s) to update; comma-separated.
    #[arg(
        short = 'a',
        long = "apps",
        value_name = "NAME[,NAME...]",
        value_delimiter = ',',
        conflicts_with_all = ["minimal_set", "configured_set"]
    )]
    pub apps: Vec<String>,

    /// GitHub token source: `prompt` to enter interactively, `load` to read from
    /// RELGET_GHB_TOKEN env var or ~/.config/relget.toml (github_token key)
    #[arg(long, default_value = "load", value_parser = ["prompt", "load"])]
    pub gh_token_source: String,

    /// Codeberg token source: `prompt` to enter interactively, `load` to read from
    /// RELGET_CDB_TOKEN env var or ~/.config/relget.toml (codeberg_token key)
    #[arg(long, default_value = "load", value_parser = ["prompt", "load"])]
    pub cb_token_source: String,

    /// GitLab token source: `prompt` to enter interactively, `load` to read from
    /// RELGET_GLB_TOKEN env var or ~/.config/relget.toml (gitlab_token key)
    #[arg(long, default_value = "load", value_parser = ["prompt", "load"])]
    pub gl_token_source: String,

    /// Update a hand-picked minimal set of apps
    #[arg(long, default_value_t = false, conflicts_with_all = ["apps", "configured_set"])]
    pub minimal_set: bool,

    /// Load a named app set from the [sets] table in ~/.config/relget.toml
    #[arg(long, value_name = "SET_NAME", conflicts_with_all = ["apps", "minimal_set"])]
    pub configured_set: Option<String>,
}

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
        conflicts_with_all = ["minimal_set", "configured_set"]
    )]
    pub apps: Vec<String>,

    /// Uninstall a hand-picked minimal set of apps
    #[arg(long, default_value_t = false, conflicts_with_all = ["apps", "configured_set"])]
    pub minimal_set: bool,

    /// Load a named app set from the [sets] table in ~/.config/relget.toml
    #[arg(long, value_name = "SET_NAME", conflicts_with_all = ["apps", "minimal_set"])]
    pub configured_set: Option<String>,
}

#[derive(Args)]
pub struct DoctorArgs {
    /// GitHub token source: `prompt` to enter interactively, `load` to read from
    /// RELGET_GHB_TOKEN env var or ~/.config/relget.toml (github_token key)
    #[arg(long, default_value = "load", value_parser = ["prompt", "load"])]
    pub gh_token_source: String,

    /// Codeberg token source: `prompt` to enter interactively, `load` to read from
    /// RELGET_CDB_TOKEN env var or ~/.config/relget.toml (codeberg_token key)
    #[arg(long, default_value = "load", value_parser = ["prompt", "load"])]
    pub cb_token_source: String,

    /// GitLab token source: `prompt` to enter interactively, `load` to read from
    /// RELGET_GLB_TOKEN env var or ~/.config/relget.toml (gitlab_token key)
    #[arg(long, default_value = "load", value_parser = ["prompt", "load"])]
    pub gl_token_source: String,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install selected apps into the prefix
    Install(InstallArgs),
    /// Print all supported app identifiers
    ListAppsIds,
    /// Print shell completion script to stdout
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
    /// Uninstall selected apps from the prefix
    ///
    /// Removes exactly the files that relget installed: the binary, all secondary binaries,
    /// man pages, and shell completions. The set of files is derived from each app's static
    /// asset descriptor, so every file relget placed is removed — no more, no less.
    #[command(verbatim_doc_comment)]
    Uninstall(UninstallArgs),
    /// Check all registry apps for potential issues against latest releases
    #[command(hide = true)]
    Doctor(DoctorArgs),
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
    Update(UpdateArgs),
}
