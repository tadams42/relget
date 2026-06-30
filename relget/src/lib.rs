mod app;
mod archive;
mod clients;
mod config;
mod prefix;
mod registry;
mod version;

pub use app::{App, Assets, Binary, ManPage, ShellCompletion};
pub use archive::ArchiveExtractor;
pub use clients::{
    CodebergClient, GithubClient, GitlabClient, RateLimitError, ReleaseMetadata, RelgetClient,
};
pub use config::Config;
pub use prefix::Prefix;
pub use registry::{AppAssetDef, AppBinaryDef, AppEntry, AssetType, CategoryEntry, Registry};
pub use version::AppVersion;
