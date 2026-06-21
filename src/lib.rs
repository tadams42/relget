mod apps;
mod archive;
mod clients;
mod config;
mod prefix;
mod version;

pub use apps::{
    App, AppAssets, AppBinary, AppEntry, CategoryInfo, Completion, ManPage, ManPagesStatus, Shell,
    ShellCompletionsStatus, all_app_entries, all_apps_identifiers, all_categories, create_app,
};
pub use archive::ArchiveExtractor;
pub use clients::{
    CodebergClient, GithubClient, GitlabClient, RateLimitError, ReleaseMetadata, RelgetClient,
};
pub use config::Config;
pub use prefix::Prefix;
pub use version::AppVersion;
