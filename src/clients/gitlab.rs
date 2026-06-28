use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::{Context, Result, anyhow};
use once_cell::sync::Lazy;
use serde_json::Value;

use super::cache::{CachedFile, ReleaseMetadata, RelgetCache};
use super::client_trait::RelgetClient;
use super::rate_limit::RateLimitError;

static CACHE: Lazy<Mutex<RelgetCache>> =
    Lazy::new(|| Mutex::new(RelgetCache::new_with_prefix("gitlab")));
static RATE_LIMITED: AtomicBool = AtomicBool::new(false);

const GL_API_URL: &str = "https://gitlab.com/api/v4/projects";

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
        self.latest_release_where(owner, repo, &|_| true)
    }

    /// Like `latest_release`, but only considers releases whose `tag_name` satisfies
    /// `tag_filter`. See [`RelgetClient::latest_release_where`] for rationale.
    fn latest_release_where(
        &self, owner: &str, repo: &str, tag_filter: &dyn Fn(&str) -> bool,
    ) -> Result<ReleaseMetadata> {
        {
            let mut cache = CACHE.lock().unwrap();
            if self.offline {
                return cache.get_release_any_age(owner, repo).ok_or_else(|| {
                    anyhow!("offline mode: no cached release for {}/{}", owner, repo)
                });
            }
            if let Some(r) = cache.get_release(owner, repo) {
                return Ok(r);
            }
        }

        if RATE_LIMITED.load(Ordering::Relaxed) {
            return Err(anyhow!(RateLimitError { site: "GitLab" }));
        }

        log::info!("app={} msg=Fetching latest GitLab release", repo);
        let encoded = format!("{}%2F{}", owner, repo);
        let url = format!("{}/{}/releases?per_page=100&page=1", GL_API_URL, encoded);

        let mut req = ureq::get(&url)
            .header("Accept", "application/json")
            .header("User-Agent", "relget");
        if let Some(token) = &self.token {
            req = req.header("PRIVATE-TOKEN", token.as_str());
        }
        let response = match req.call() {
            Ok(r) => r,
            Err(ureq::Error::StatusCode(429)) => {
                RATE_LIMITED.store(true, Ordering::Relaxed);
                return Err(anyhow!(RateLimitError { site: "GitLab" }));
            }
            Err(e) => {
                return Err(anyhow::Error::from(e)).with_context(|| {
                    format!("Can't fetch GitLab release info for {}/{}", owner, repo)
                });
            }
        };

        let releases: Vec<Value> = response
            .into_body()
            .read_json()
            .with_context(|| format!("Invalid JSON from GitLab for {}/{}", owner, repo))?;

        let data = releases
            .into_iter()
            .find(|r| {
                r["assets"]["links"]
                    .as_array()
                    .map(|a| !a.is_empty())
                    .unwrap_or(false)
                    && !r["upcoming_release"].as_bool().unwrap_or(false)
                    && r["tag_name"].as_str().is_none_or(&tag_filter)
            })
            .ok_or_else(|| anyhow!("No release with assets for {}/{}", owner, repo))?;

        let normalized = normalize_gitlab_release(data);
        let release = ReleaseMetadata::new(owner, repo, normalized)?;
        CACHE.lock().unwrap().store_release(release.clone())?;
        Ok(release)
    }

    fn download_asset(&self, owner: &str, repo: &str, name: &str) -> Result<CachedFile> {
        if RATE_LIMITED.load(Ordering::Relaxed) {
            return Err(anyhow!(RateLimitError { site: "GitLab" }));
        }

        let release = self.latest_release(owner, repo)?;

        let asset_id = release
            .asset_id(name)
            .ok_or_else(|| anyhow!("No such asset '{}' in {}/{}", name, owner, repo))?;

        {
            let mut cache = CACHE.lock().unwrap();
            if let Some(a) = cache.get_asset(owner, repo, name, asset_id) {
                return Ok(a);
            }
        }

        if self.offline {
            return Err(anyhow!(
                "offline mode: no cached asset '{}' for {}/{}",
                name,
                owner,
                repo
            ));
        }

        let url = release
            .asset_download_url(name)
            .ok_or_else(|| anyhow!("No download URL for asset '{}' in {}/{}", name, owner, repo))?;

        if !url.starts_with("http:") && !url.starts_with("https:") {
            return Err(anyhow!("Unsafe URL scheme: {}", url));
        }

        log::info!("app={} msg=Downloading {}", repo, name);
        let mut req = ureq::get(&url).header("User-Agent", "relget");
        if let Some(token) = &self.token {
            req = req.header("PRIVATE-TOKEN", token.as_str());
        }
        let buf = req
            .call()
            .with_context(|| format!("Couldn't download '{}' from GitLab", name))?
            .into_body()
            .into_with_config()
            .limit(500 * 1024 * 1024)
            .read_to_vec()
            .with_context(|| format!("Couldn't read downloaded asset '{}'", name))?;
        log::info!("app={} msg=Downloaded {}", repo, name);

        let asset = CachedFile {
            api_id: asset_id,
            owner:  owner.to_string(),
            repo:   repo.to_string(),
            name:   name.to_string(),
            data:   buf,
        };
        CACHE.lock().unwrap().store_asset(asset.clone())?;
        Ok(asset)
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
