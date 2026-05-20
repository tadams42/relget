use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete::generate;

mod cli;
use cli::{Cli, Commands};

use relget::{
    install_apps, known_apps_identifiers, load_codeberg_token, load_github_token, select_apps,
    uninstall_apps,
};

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(|buf, record| {
            use std::io::Write;
            writeln!(buf, "lvl={} app=installer msg={}", record.level(), record.args())
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
            generate(shell, &mut Cli::command(), "relget", &mut std::io::stdout());
        }
        Some(Commands::Uninstall) => {
            let selected = select_apps(&cli.apps, cli.minimal_set)?;
            let removed = uninstall_apps(&cli.prefix, &selected)?;
            if removed.is_empty() {
                println!("No files removed.");
            } else {
                println!("Removed files:");
                for path in removed {
                    println!("- {}", path.display());
                }
            }
        }
        Some(Commands::Reinstall) => {
            let selected = select_apps(&cli.apps, cli.minimal_set)?;
            let removed = uninstall_apps(&cli.prefix, &selected)?;
            if removed.is_empty() {
                println!("No files removed.");
            } else {
                println!("Removed files:");
                for path in &removed {
                    println!("- {}", path.display());
                }
            }
            log::info!("Reinstalling into: {:?}", cli.prefix);
            let gh_token = load_github_token(&cli.gh_token_source)?;
            let cb_token = load_codeberg_token(&cli.cb_token_source)?;
            let installed = install_apps(&cli.prefix, &selected, gh_token, cb_token)?;
            if !installed.is_empty() {
                println!("Installed files:");
                for path in installed {
                    println!("- {}", path.display());
                }
            }
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
