use std::sync::atomic::AtomicBool;
use std::sync::{Arc, LazyLock, Mutex};

use anyhow::Result;

use super::cache::{CachedFile, ReleaseMetadata, RelgetCache};
use super::client_trait::RelgetClient;
use super::forge::Forge;

static CACHE: LazyLock<Mutex<RelgetCache>> = LazyLock::new(|| Mutex::new(RelgetCache::new()));
static RATE_LIMITED: AtomicBool = AtomicBool::new(false);

static GITHUB: Forge = Forge {
    site:               "GitHub",
    cache:              &CACHE,
    rate_limited:       &RATE_LIMITED,
    accept:             "application/vnd.github+json",
    // GitHub reports rate limiting as 403 as well as 429
    rate_limit_codes:   &[429, 403],
    releases_url:       |owner, repo| {
        format!("https://api.github.com/repos/{owner}/{repo}/releases?per_page=100&page=1")
    },
    auth_header:        |token| ("Authorization", format!("Bearer {token}")),
    // browser_download_url is publicly fetchable and may redirect to a CDN
    auth_on_download:   false,
    release_ok:         |r| r["assets"].as_array().is_some_and(|a| !a.is_empty()),
    normalize:          |data| data,
    default_tag_filter: |tag| tag != "nightly",
    source_tarball:     true,
};

pub struct GithubClient {
    pub token:   Option<String>,
    pub offline: bool,
}

impl GithubClient {
    pub fn new(token: Option<String>, offline: bool) -> Self { Self { token, offline } }
}

impl RelgetClient for GithubClient {
    fn latest_release(&self, owner: &str, repo: &str) -> Result<ReleaseMetadata> {
        GITHUB.latest_release(self.token.as_deref(), self.offline, owner, repo)
    }

    fn latest_release_where(
        &self, owner: &str, repo: &str, tag_filter: &dyn Fn(&str) -> bool,
    ) -> Result<ReleaseMetadata> {
        GITHUB.latest_release_where(self.token.as_deref(), self.offline, owner, repo, tag_filter)
    }

    fn download_asset(&self, owner: &str, repo: &str, name: &str) -> Result<Arc<CachedFile>> {
        GITHUB.download_asset(self.token.as_deref(), self.offline, owner, repo, name)
    }
}
