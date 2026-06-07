#[allow(clippy::module_inception)]
mod cli;
mod doctor;
mod helpers;
mod install;
mod list;
mod sync;
mod uninstall;
mod update;

pub use cli::{create_cli, execute_cli};
