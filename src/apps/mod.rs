mod app_trait;
mod apps_factory;
mod apps_registry;
mod containers;
mod data;
mod databases;
mod dev_envs;
mod dev_tools;
mod files;
mod git;
mod http;
mod logs;
mod music;
mod other;
mod shell;

pub use app_trait::App;
pub use apps_factory::create_app;
pub use apps_registry::{all_app_entries, all_apps_identifiers, minimal_set_identifiers};
