use anyhow::Result;

use super::cache::{CachedFile, ReleaseMetadata};

/// Shared interface for all forge clients (GitHub, GitLab, Codeberg).
///
/// Each implementation handles provider-specific auth, URL encoding, and JSON normalisation
/// internally; callers receive a uniform [`ReleaseMetadata`] regardless of which forge hosts
/// the app.
///
/// # Object safety
///
/// The trait is object-safe and intended to be used as `Arc<dyn RelgetClient>`, allowing apps to
/// remain provider-agnostic: `apps_factory` constructs the right concrete client and passes it
/// through the trait object so no app struct needs to know which forge it talks to.
pub trait RelgetClient {
    /// Fetches the latest non-nightly release for `owner/repo`.
    fn latest_release(&self, owner: &str, repo: &str) -> Result<ReleaseMetadata>;

    /// Like [`RelgetClient::latest_release`], but only considers releases whose `tag_name`
    /// satisfies `tag_filter`.
    ///
    /// Use this when a repo mixes stable and nightly/dated releases under different tag schemes
    /// (e.g. rust-analyzer uses `v0.3.x` for stable and `YYYY-MM-DD` for nightly). Fetches up to
    /// 100 releases to survive long nightly streaks.
    fn latest_release_where(
        &self, owner: &str, repo: &str, tag_filter: &dyn Fn(&str) -> bool,
    ) -> Result<ReleaseMetadata>;

    /// Downloads a named release asset for `owner/repo`, returning its bytes.
    ///
    /// Pass `"tarball"` as `name` to download the source tarball instead of a release asset.
    fn download_asset(&self, owner: &str, repo: &str, name: &str) -> Result<CachedFile>;
}
