use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use std::path::PathBuf;

use rutils_downloader::{
    install_apps, known_apps_identifiers, load_codeberg_token, load_github_token, select_apps,
    DEFAULT_PREFIX,
};

#[derive(Parser)]
#[command(name = "rutils-downloader")]
#[command(about = "Installs or updates CLI utilities directly from GitHub releases")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Install prefix (e.g. /usr/local or ~/.local)
    #[arg(short = 'p', long, default_value = DEFAULT_PREFIX)]
    prefix: PathBuf,

    /// App(s) to install; may be repeated. Defaults to all apps.
    #[arg(short = 'a', long = "apps", value_name = "NAME")]
    apps: Vec<String>,

    /// Where to load GitHub API token from (prompt or load)
    #[arg(long, default_value = "prompt", value_parser = ["prompt", "load"])]
    gh_token_source: String,

    /// Where to load Codeberg API token from (prompt or load)
    #[arg(long, default_value = "load", value_parser = ["prompt", "load"])]
    cb_token_source: String,

    /// Install a hand-picked minimal set of apps (overrides --apps)
    #[arg(long, default_value_t = false)]
    minimal_set: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Print all supported app identifiers
    ListAppsIds,
    /// Print shell completion script to stdout
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info"),
    )
    .format(|buf, record| {
        use std::io::Write;
        writeln!(
            buf,
            "lvl={} app=installer msg={}",
            record.level(),
            record.args()
        )
    })
    .init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::ListAppsIds) => {
            for id in known_apps_identifiers() {
                println!("{}", id);
            }
        }
        Some(Commands::Completions { shell }) => {
            generate(shell, &mut Cli::command(), "rutils-downloader", &mut std::io::stdout());
        }
        None => {
            log::info!("Installing into: {:?}", cli.prefix);
            let gh_token = load_github_token(&cli.gh_token_source)?;
            let cb_token = load_codeberg_token(&cli.cb_token_source)?;
            let selected = select_apps(&cli.apps, cli.minimal_set)?;
            let installed = install_apps(&cli.prefix, &selected, gh_token, cb_token)?;
            if !installed.is_empty() {
                println!("Installed files:");
                for path in installed {
                    println!("- {}", path.display());
                }
            }
        }
    }

    Ok(())
}
