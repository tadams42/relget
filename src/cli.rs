use clap::{Parser, Subcommand};
use clap_complete::Shell;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rutils-downloader")]
#[command(version)]
#[command(about = "Installs or updates CLI utilities directly from GitHub releases")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Install prefix (e.g. /usr/local or ~/.local)
    #[arg(short = 'p', long, default_value = "/usr/local")]
    pub prefix: PathBuf,

    /// App(s) to install; may be repeated. Defaults to all apps.
    #[arg(short = 'a', long = "apps", value_name = "NAME")]
    pub apps: Vec<String>,

    /// Where to load GitHub API token from (prompt or load)
    #[arg(long, default_value = "prompt", value_parser = ["prompt", "load"])]
    pub gh_token_source: String,

    /// Where to load Codeberg API token from (prompt or load)
    #[arg(long, default_value = "load", value_parser = ["prompt", "load"])]
    pub cb_token_source: String,

    /// Install a hand-picked minimal set of apps (overrides --apps)
    #[arg(long, default_value_t = false)]
    pub minimal_set: bool,
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
}
