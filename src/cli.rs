use clap::{Parser, Subcommand};
use clap_complete::Shell;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "relget")]
#[command(version)]
#[command(about = "Installs or updates CLI utilities directly from GitHub releases")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Install prefix (e.g. /usr/local or ~/.local)
    #[arg(short = 'p', long, default_value = "/usr/local", global = true)]
    pub prefix: PathBuf,

    /// App(s) to install; comma-separated. Defaults to all apps.
    #[arg(short = 'a', long = "apps", value_name = "NAME[,NAME...]", value_delimiter = ',', global = true)]
    pub apps: Vec<String>,

    /// Where to load GitHub API token from (prompt or load)
    #[arg(long, default_value = "load", value_parser = ["prompt", "load"], global = true)]
    pub gh_token_source: String,

    /// Where to load Codeberg API token from (prompt or load)
    #[arg(long, default_value = "load", value_parser = ["prompt", "load"], global = true)]
    pub cb_token_source: String,

    /// Install a hand-picked minimal set of apps (overrides --apps)
    #[arg(long, default_value_t = false, global = true)]
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
    /// Uninstall selected apps from the given prefix (best-effort)
    ///
    /// Removes the binary, shell completion files, and man pages that match
    /// the app's executable name under the given prefix. This is a
    /// best-effort operation:
    ///
    ///   - Only files that follow relget's standard installation layout are
    ///     searched. Files placed elsewhere will not be touched.
    ///
    ///   - Apps that install multiple binaries under different names (e.g.
    ///     `uv` also installs `uvx`) will have only the primary binary
    ///     removed. The secondary binaries and their completions are left
    ///     behind.
    ///
    ///   - Man pages with a separator other than `-` (e.g. `eza_colors.5`)
    ///     are not matched and will not be removed.
    ///
    /// Token flags (--gh-token-source, --cb-token-source) are accepted but
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
