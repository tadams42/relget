use std::sync::atomic::AtomicBool;
use std::sync::{Arc, LazyLock, Mutex};

use anyhow::Result;

use super::cache::{CachedFile, ReleaseMetadata, RelgetCache};
use super::client_trait::RelgetClient;
use super::forge::Forge;

static CACHE: LazyLock<Mutex<RelgetCache>> =
    LazyLock::new(|| Mutex::new(RelgetCache::new_with_prefix("codeberg")));
static RATE_LIMITED: AtomicBool = AtomicBool::new(false);

static CODEBERG: Forge = Forge {
    site:               "Codeberg",
    cache:              &CACHE,
    rate_limited:       &RATE_LIMITED,
    accept:             "application/json",
    rate_limit_codes:   &[429],
    releases_url:       |owner, repo| {
        format!("https://codeberg.org/api/v1/repos/{owner}/{repo}/releases?limit=100&page=1")
    },
    auth_header:        |token| ("Authorization", format!("token {token}")),
    auth_on_download:   true,
    release_ok:         |r| {
        r["assets"].as_array().is_some_and(|a| !a.is_empty())
            && !r["draft"].as_bool().unwrap_or(false)
            && !r["prerelease"].as_bool().unwrap_or(false)
    },
    normalize:          |data| data,
    default_tag_filter: |_| true,
    source_tarball:     false,
};

pub struct CodebergClient {
    pub token:   Option<String>,
    pub offline: bool,
}

impl CodebergClient {
    pub fn new(token: Option<String>, offline: bool) -> Self { Self { token, offline } }
}

impl RelgetClient for CodebergClient {
    fn latest_release(&self, owner: &str, repo: &str) -> Result<ReleaseMetadata> {
        CODEBERG.latest_release(self.token.as_deref(), self.offline, owner, repo)
    }

    fn latest_release_where(
        &self, owner: &str, repo: &str, tag_filter: &dyn Fn(&str) -> bool,
    ) -> Result<ReleaseMetadata> {
        CODEBERG.latest_release_where(self.token.as_deref(), self.offline, owner, repo, tag_filter)
    }

    fn download_asset(&self, owner: &str, repo: &str, name: &str) -> Result<Arc<CachedFile>> {
        CODEBERG.download_asset(self.token.as_deref(), self.offline, owner, repo, name)
    }
}
