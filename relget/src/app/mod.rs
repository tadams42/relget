#[allow(clippy::module_inception)]
mod app;
mod assets;

pub use app::App;
pub use assets::{Assets, Binary, ManPage, ShellCompletion};
