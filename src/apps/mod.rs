mod app_trait;
mod apps_factory;
mod apps_registry;
mod coding;
mod containers;
mod data_processing;
mod dev_envs;
mod docs_diag;
mod encryption;
mod files;
mod git;
mod http;
mod logs;
mod networking;
mod shell;
mod system;

pub use app_trait::App;
pub use apps_factory::create_app;
pub use apps_registry::{
    AppEntry, CategoryInfo, ManPagesStatus, ShellCompletionsStatus, all_app_entries,
    all_apps_identifiers, all_categories,
};

pub(in crate::apps) use app_trait::{
    gen_completions_shell_flag, gen_completions_subcommand, gen_completions_with_shell_arg,
    run_cmd, with_temp_exe,
};
