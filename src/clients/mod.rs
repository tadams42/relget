mod cache;
mod client_trait;
mod codeberg;
mod github;
mod gitlab;
mod rate_limit;

pub use cache::ReleaseMetadata;
pub use client_trait::RelgetClient;
pub use codeberg::CodebergClient;
pub use github::GithubClient;
pub use gitlab::GitlabClient;
pub use rate_limit::RateLimitError;
