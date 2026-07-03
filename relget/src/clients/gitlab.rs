use std::sync::atomic::AtomicBool;
use std::sync::{Arc, LazyLock, Mutex};

use anyhow::Result;
use serde_json::Value;

use super::cache::{CachedFile, ReleaseMetadata, RelgetCache};
use super::client_trait::RelgetClient;
use super::forge::Forge;

static CACHE: LazyLock<Mutex<RelgetCache>> =
    LazyLock::new(|| Mutex::new(RelgetCache::new_with_prefix("gitlab")));
static RATE_LIMITED: AtomicBool = AtomicBool::new(false);

static GITLAB: Forge = Forge {
    site:               "GitLab",
    cache:              &CACHE,
    rate_limited:       &RATE_LIMITED,
    accept:             "application/json",
    rate_limit_codes:   &[429],
    releases_url:       |owner, repo| {
        // GitLab addresses projects by URL-encoded "{owner}/{repo}"
        format!("https://gitlab.com/api/v4/projects/{owner}%2F{repo}/releases?per_page=100&page=1")
    },
    auth_header:        |token| ("PRIVATE-TOKEN", token.to_string()),
    auth_on_download:   true,
    release_ok:         |r| {
        r["assets"]["links"]
            .as_array()
            .is_some_and(|a| !a.is_empty())
            && !r["upcoming_release"].as_bool().unwrap_or(false)
    },
    normalize:          normalize_gitlab_release,
    default_tag_filter: |_| true,
    source_tarball:     false,
};

/// API client for GitLab releases.
///
/// GitLab stores release assets under `assets.links[].{ id, name, direct_asset_url }` in the API
/// response. `GitlabClient` normalizes these to the same shape as GitHub/Codeberg before storing
/// in [`ReleaseMetadata`], so the same `release.find_asset()` / download helpers work across all
/// three forges.
pub struct GitlabClient {
    pub token:   Option<String>,
    pub offline: bool,
}

impl GitlabClient {
    pub fn new(token: Option<String>, offline: bool) -> Self { Self { token, offline } }
}

impl RelgetClient for GitlabClient {
    fn latest_release(&self, owner: &str, repo: &str) -> Result<ReleaseMetadata> {
        GITLAB.latest_release(self.token.as_deref(), self.offline, owner, repo)
    }

    fn latest_release_where(
        &self, owner: &str, repo: &str, tag_filter: &dyn Fn(&str) -> bool,
    ) -> Result<ReleaseMetadata> {
        GITLAB.latest_release_where(self.token.as_deref(), self.offline, owner, repo, tag_filter)
    }

    fn download_asset(&self, owner: &str, repo: &str, name: &str) -> Result<Arc<CachedFile>> {
        GITLAB.download_asset(self.token.as_deref(), self.offline, owner, repo, name)
    }
}

/// Normalize GitLab release JSON to match the GitHub/Codeberg shape expected by
/// `ReleaseMetadata`.
///
/// GitLab stores assets at `assets.links[].{id, name, direct_asset_url}`.
/// `ReleaseMetadata` methods expect `assets[].{id, name, browser_download_url}`.
fn normalize_gitlab_release(mut data: Value) -> Value {
    let links = data["assets"]["links"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let normalized: Vec<Value> = links
        .into_iter()
        .map(|mut link| {
            let url = link["direct_asset_url"].as_str().map(|s| s.to_string());
            if let (Some(obj), Some(url)) = (link.as_object_mut(), url) {
                obj.insert("browser_download_url".to_string(), Value::String(url));
            }
            link
        })
        .collect();

    if let Some(obj) = data.as_object_mut() {
        obj.insert("assets".to_string(), Value::Array(normalized));
    }
    data
}
