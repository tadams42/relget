use anyhow::{Context, Result, anyhow};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use super::cache::{CachedFile, ReleaseMetadata, RelgetCache};
use super::rate_limit::RateLimitError;

static CACHE: Lazy<Mutex<RelgetCache>> = Lazy::new(|| Mutex::new(RelgetCache::new()));
static RATE_LIMITED: AtomicBool = AtomicBool::new(false);

const GH_API_URL: &str = "https://api.github.com/repos";

pub struct GithubClient {
    pub token:   Option<String>,
    pub offline: bool,
}

impl GithubClient {
    pub fn new(token: Option<String>, offline: bool) -> Self { Self { token, offline } }

    pub fn latest_release(&self, owner: &str, repo: &str) -> Result<ReleaseMetadata> {
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
            return Err(anyhow!(RateLimitError { site: "GitHub" }));
        }

        log::info!("app={} msg=Fetching latest GitHub release", repo);
        let url = format!("{}/{}/{}/releases?per_page=5&page=1", GH_API_URL, owner, repo);

        let mut req = ureq::get(&url)
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "relget");
        if let Some(token) = &self.token {
            req = req.header("Authorization", &format!("Bearer {}", token));
        }
        let response = match req.call() {
            Ok(r) => r,
            Err(ureq::Error::StatusCode(429 | 403)) => {
                RATE_LIMITED.store(true, Ordering::Relaxed);
                return Err(anyhow!(RateLimitError { site: "GitHub" }));
            }
            Err(e) => {
                return Err(anyhow::Error::from(e)).with_context(|| {
                    format!("Can't fetch GitHub release info for {}/{}", owner, repo)
                });
            }
        };

        let releases: Vec<serde_json::Value> = response
            .into_body()
            .read_json()
            .with_context(|| format!("Invalid JSON from GitHub for {}/{}", owner, repo))?;

        let data = releases
            .into_iter()
            .find(|r| {
                r["assets"]
                    .as_array()
                    .map(|a| !a.is_empty())
                    .unwrap_or(false)
                    && r["tag_name"].as_str() != Some("nightly")
            })
            .ok_or_else(|| anyhow!("No release with assets for {}/{}", owner, repo))?;

        let release = ReleaseMetadata::new(owner, repo, data)?;
        CACHE.lock().unwrap().store_release(release.clone())?;
        Ok(release)
    }

    pub fn download_asset(&self, owner: &str, repo: &str, name: &str) -> Result<CachedFile> {
        if RATE_LIMITED.load(Ordering::Relaxed) {
            return Err(anyhow!(RateLimitError { site: "GitHub" }));
        }

        let release = self.latest_release(owner, repo)?;

        let api_id = if name == "tarball" {
            release.api_id().unwrap_or(0)
        } else {
            release
                .asset_id(name)
                .ok_or_else(|| anyhow!("No such asset '{}' in {}/{}", name, owner, repo))?
        };

        {
            let mut cache = CACHE.lock().unwrap();
            if let Some(a) = cache.get_asset(owner, repo, name, api_id) {
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

        let url = if name == "tarball" {
            release
                .tarball_url()
                .ok_or_else(|| anyhow!("No tarball URL for {}/{}", owner, repo))?
        } else {
            release.asset_download_url(name).ok_or_else(|| {
                anyhow!("No download URL for asset '{}' in {}/{}", name, owner, repo)
            })?
        };

        if !url.starts_with("http:") && !url.starts_with("https:") {
            return Err(anyhow!("Unsafe URL scheme: {}", url));
        }

        log::info!("app={} msg=Downloading {}", repo, name);
        let resp = ureq::get(&url)
            .header("User-Agent", "relget")
            .call()
            .with_context(|| format!("Couldn't download '{}' from GitHub", name))?;

        let buf = resp
            .into_body()
            .into_with_config()
            .limit(500 * 1024 * 1024)
            .read_to_vec()
            .with_context(|| format!("Couldn't read downloaded asset '{}'", name))?;
        log::info!("app={} msg=Downloaded {}", repo, name);

        let asset = CachedFile {
            api_id,
            owner: owner.to_string(),
            repo: repo.to_string(),
            name: name.to_string(),
            data: buf,
        };
        CACHE.lock().unwrap().store_asset(asset.clone())?;
        Ok(asset)
    }
}
