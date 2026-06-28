mod app_assets;
mod app_trait;
mod apps_factory;
mod coding;
mod generic_app;

pub use app_assets::{AppAssets, AppBinary, Completion, ManPage, Shell};
pub use app_trait::App;
pub use apps_factory::create_app;
