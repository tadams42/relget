mod app_entry;
mod category_entry;
mod doctor;
mod loader;
#[allow(clippy::module_inception)]
mod registry;

pub use app_entry::{
    AppAssetDef, AppBinaryDef, AppEntry, AssetType, CompletionSource, ManPageDef,
    ShellCompletionDef, ShellKind,
};
pub use category_entry::CategoryEntry;
pub use registry::Registry;
