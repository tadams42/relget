mod app_assets;
mod app_trait;
mod apps_factory;
mod coding;
mod generic_app;
mod http;

pub use app_assets::{AppAssets, AppBinary, Completion, ManPage, Shell};
pub use app_trait::App;
pub(crate) use app_trait::{gen_completions_subcommand, run_cmd, with_temp_exe};
pub use apps_factory::create_app;
