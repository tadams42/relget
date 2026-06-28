use std::path::PathBuf;

use anyhow::Result;
use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::{Args, CommandFactory, FromArgMatches, Parser, Subcommand};
use clap_complete::{Shell, generate};
use relget::{Prefix, Registry};

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(|buf, record| {
            use std::io::Write;
            writeln!(buf, "lvl={} {}", record.level(), record.args())
        })
        .init();
    let cli = create_cli()?;
    execute_cli(&cli)
}

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
    let mut cmd = Cli::command().styles(styles());

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
        Commands::Sync(args) => sync_command(args, cli.offline)?,
        Commands::Completions { shell } => {
            generate(*shell, &mut Cli::command(), "relget", &mut std::io::stdout())
        }
        Commands::Registry(args) => registry_command(args, cli.offline)?,
    }

    Ok(())
}

#[derive(Args)]
pub struct UpdateArgs {
    /// Install prefix (e.g. /usr/local or ~/.local)
    #[arg(short = 'p', long, default_value = Prefix::DEFAULT_PREFIX)]
    pub prefix: PathBuf,

    /// App(s) to update; comma-separated.
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

pub fn update_command(args: &UpdateArgs, offline: bool) -> Result<()> {
    let prefix = Prefix::new(args.prefix.clone());
    prefix.update(&args.apps, args.configured_set.as_deref(), offline)
}

#[derive(Args)]
pub struct SyncArgs {
    /// Install prefix (e.g. /usr/local or ~/.local)
    #[arg(short = 'p', long, default_value = Prefix:: DEFAULT_PREFIX)]
    pub prefix: PathBuf,

    /// App(s) to sync; comma-separated.
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

pub fn sync_command(args: &SyncArgs, offline: bool) -> Result<()> {
    let prefix = Prefix::new(args.prefix.clone());
    prefix.sync(&args.apps, args.configured_set.as_deref(), offline)
}

#[derive(Args)]
pub struct UninstallArgs {
    /// Install prefix (e.g. /usr/local or ~/.local)
    #[arg(short = 'p', long, default_value = Prefix::DEFAULT_PREFIX)]
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
    let prefix = Prefix::new(args.prefix.clone());
    prefix.uninstall(&args.apps, args.configured_set.as_deref())
}

#[derive(Args)]
pub struct InstallArgs {
    /// Install prefix (e.g. /usr/local or ~/.local)
    #[arg(short = 'p', long, default_value = Prefix::DEFAULT_PREFIX)]
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

    #[arg(
        long,
        value_name = "SET_NAME",
        conflicts_with_all = ["apps"],
        long_help = "Load a named app set from the [sets] table in ~/.config/relget.toml"
    )]
    pub configured_set: Option<String>,
}

pub fn install_apps_command(args: &InstallArgs, offline: bool) -> Result<()> {
    let prefix = Prefix::new(args.prefix.clone());
    prefix.install(&args.apps, args.configured_set.as_deref(), offline)
}

#[derive(Args)]
pub struct DoctorArgs {}

#[derive(Args)]
pub struct RegistryArgs {
    #[command(subcommand)]
    pub command: RegistrySubcommands,
}

#[derive(Subcommand)]
pub enum RegistrySubcommands {
    /// Validate all registry JSON files against their schemas
    Validate,
    /// Print all supported app identifiers
    ListAppsIds,
    /// Check all registry apps for potential issues against latest releases
    #[command(hide = true)]
    Doctor(DoctorArgs),
}

pub fn registry_command(args: &RegistryArgs, offline: bool) -> Result<()> {
    match &args.command {
        RegistrySubcommands::Validate => {
            let registry = Registry::load()?;
            registry.validate()?;
            println!("Registry valid. {} apps validated.", registry.apps.len());
        }
        RegistrySubcommands::ListAppsIds => {
            for id in Registry::global().identifiers() {
                println!("{}", id);
            }
        }
        RegistrySubcommands::Doctor(_) => Registry::global().doctor(offline)?,
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

#[derive(Subcommand)]
pub enum Commands {
    /// Install selected apps into the prefix
    Install(InstallArgs),
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
    /// Reconcile installed apps with a selected set
    ///
    /// Installs apps in the set that are not yet present in the prefix, and uninstalls apps
    /// that are present in the prefix but not in the set. Either --apps or --configured-set
    /// must be specified.
    #[command(verbatim_doc_comment)]
    Sync(SyncArgs),
    /// Update relget-managed apps in the prefix
    ///
    /// Without selectors: scans `<prefix>/bin/` for executables that match a known app in the
    /// registry and updates each one. When a binary name matches more than one registry entry
    /// (e.g. `qsv` for both `qsv` and `qsv-all`), the first alphabetical match is used and a
    /// warning is printed.
    ///
    /// With --apps / --configured-set: updates only the specified apps,
    /// regardless of whether they are currently installed.
    ///
    /// Apps already at the latest version are skipped in both cases.
    #[command(verbatim_doc_comment)]
    Update(UpdateArgs),
    /// Work with the declarative JSON-based app registry
    Registry(RegistryArgs),
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- mutual exclusion ---

    #[test]
    fn install_apps_and_configured_set_conflict() {
        let result =
            Cli::try_parse_from(["relget", "install", "--apps", "rg", "--configured-set", "s"]);
        assert!(result.is_err());
    }

    #[test]
    fn update_apps_and_configured_set_conflict() {
        let result =
            Cli::try_parse_from(["relget", "update", "--apps", "rg", "--configured-set", "s"]);
        assert!(result.is_err());
    }

    #[test]
    fn uninstall_apps_and_configured_set_conflict() {
        let result = Cli::try_parse_from([
            "relget",
            "uninstall",
            "--apps",
            "rg",
            "--configured-set",
            "s",
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn sync_apps_and_configured_set_conflict() {
        let result =
            Cli::try_parse_from(["relget", "sync", "--apps", "rg", "--configured-set", "s"]);
        assert!(result.is_err());
    }

    // --- successful parses ---

    #[test]
    fn install_parses_comma_separated_apps() {
        let cli = Cli::try_parse_from(["relget", "install", "--apps", "rg,bat"]).unwrap();
        if let Commands::Install(args) = cli.command {
            assert_eq!(args.apps, ["rg", "bat"]);
        } else {
            panic!("expected Install");
        }
    }

    #[test]
    fn update_allows_no_selectors() {
        let result = Cli::try_parse_from(["relget", "update"]);
        assert!(result.is_ok());
    }

    #[test]
    fn sync_parses_apps() {
        let cli = Cli::try_parse_from(["relget", "sync", "--apps", "rg"]).unwrap();
        if let Commands::Sync(args) = cli.command {
            assert_eq!(args.apps, ["rg"]);
        } else {
            panic!("expected Sync");
        }
    }

    #[test]
    fn sync_parses_configured_set() {
        let cli = Cli::try_parse_from(["relget", "sync", "--configured-set", "mysets"]).unwrap();
        if let Commands::Sync(args) = cli.command {
            assert_eq!(args.configured_set.as_deref(), Some("mysets"));
        } else {
            panic!("expected Sync");
        }
    }

    #[test]
    fn offline_flag_is_global() {
        let cli = Cli::try_parse_from(["relget", "--offline", "install", "--apps", "rg"]).unwrap();
        assert!(cli.offline);
    }

    #[test]
    fn prefix_defaults_to_usr_local() {
        let cli = Cli::try_parse_from(["relget", "install", "--apps", "rg"]).unwrap();
        if let Commands::Install(args) = cli.command {
            assert_eq!(args.prefix.to_str().unwrap(), "/usr/local");
        } else {
            panic!("expected Install");
        }
    }

    #[test]
    fn custom_prefix_is_accepted() {
        let cli =
            Cli::try_parse_from(["relget", "install", "--apps", "rg", "--prefix", "/tmp/test"])
                .unwrap();
        if let Commands::Install(args) = cli.command {
            assert_eq!(args.prefix.to_str().unwrap(), "/tmp/test");
        } else {
            panic!("expected Install");
        }
    }
}
