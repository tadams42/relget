//! Shared release-fetch / asset-download engine used by all forge clients.
//!
//! GitHub, GitLab, and Codeberg follow the exact same flow: consult the cache, honor the
//! rate-limit flag, hit the releases API, pick the newest acceptable release, download and
//! cache assets. Everything that actually differs between providers is captured in a
//! [`Forge`] value; the flow itself lives here once.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};

use anyhow::{Context, Result, anyhow};
use serde_json::Value;

use super::cache::{CachedFile, ReleaseMetadata, RelgetCache};
use super::rate_limit::RateLimitError;

const DOWNLOAD_LIMIT: u64 = 500 * 1024 * 1024;

/// Static description of one forge: the provider-specific pieces of the shared flow.
pub(super) struct Forge {
    /// Human-readable name used in log and error messages ("GitHub").
    pub site:               &'static str,
    /// Process-wide cache singleton for this forge.
    pub cache:              &'static LazyLock<Mutex<RelgetCache>>,
    /// Process-wide "we got rate limited" flag for this forge.
    pub rate_limited:       &'static AtomicBool,
    /// `Accept` header for the releases API.
    pub accept:             &'static str,
    /// HTTP status codes this forge uses to signal rate limiting.
    pub rate_limit_codes:   &'static [u16],
    /// Builds the releases listing URL.
    pub releases_url:       fn(owner: &str, repo: &str) -> String,
    /// Auth header `(name, value)` for a token.
    pub auth_header:        fn(token: &str) -> (&'static str, String),
    /// Whether asset downloads need the auth header (GitHub asset URLs are public).
    pub auth_on_download:   bool,
    /// Provider-specific release acceptance test (assets present, not draft, ...).
    pub release_ok:         fn(&Value) -> bool,
    /// Normalizes provider JSON into the GitHub shape (identity for GitHub/Codeberg).
    pub normalize:          fn(Value) -> Value,
    /// Tag filter applied by plain `latest_release` (GitHub skips `nightly`).
    pub default_tag_filter: fn(&str) -> bool,
    /// Whether the `"tarball"` sentinel (source tarball download) is supported.
    pub source_tarball:     bool,
}

impl Forge {
    pub fn latest_release(
        &self, token: Option<&str>, offline: bool, owner: &str, repo: &str,
    ) -> Result<ReleaseMetadata> {
        self.latest_release_where(token, offline, owner, repo, &self.default_tag_filter)
    }

    pub fn latest_release_where(
        &self, token: Option<&str>, offline: bool, owner: &str, repo: &str,
        tag_filter: &dyn Fn(&str) -> bool,
    ) -> Result<ReleaseMetadata> {
        {
            let mut cache = self.cache.lock().unwrap();
            if offline {
                return cache
                    .get_release_any_age(owner, repo)
                    .ok_or_else(|| anyhow!("offline mode: no cached release for {owner}/{repo}"));
            }
            if let Some(r) = cache.get_release(owner, repo) {
                return Ok(r);
            }
        }

        if self.rate_limited.load(Ordering::Relaxed) {
            return Err(anyhow!(RateLimitError { site: self.site }));
        }

        log::info!("app={} msg=Fetching latest {} release metadata", repo, self.site);
        let url = (self.releases_url)(owner, repo);

        let mut req = ureq::get(&url)
            .header("Accept", self.accept)
            .header("User-Agent", "relget");
        if let Some(token) = token {
            let (name, value) = (self.auth_header)(token);
            req = req.header(name, &value);
        }
        let response = match req.call() {
            Ok(r) => r,
            Err(ureq::Error::StatusCode(code)) if self.rate_limit_codes.contains(&code) => {
                self.rate_limited.store(true, Ordering::Relaxed);
                return Err(anyhow!(RateLimitError { site: self.site }));
            }
            Err(e) => {
                return Err(anyhow::Error::from(e)).with_context(|| {
                    format!("Can't fetch {} release info for {}/{}", self.site, owner, repo)
                });
            }
        };

        let releases: Vec<Value> = response
            .into_body()
            .read_json()
            .with_context(|| format!("Invalid JSON from {} for {}/{}", self.site, owner, repo))?;

        let data = releases
            .into_iter()
            .find(|r| (self.release_ok)(r) && r["tag_name"].as_str().is_none_or(tag_filter))
            .ok_or_else(|| anyhow!("No release with assets for {owner}/{repo}"))?;

        let release = ReleaseMetadata::new(owner, repo, (self.normalize)(data));
        self.cache.lock().unwrap().store_release(release.clone())?;
        Ok(release)
    }

    pub fn download_asset(
        &self, token: Option<&str>, offline: bool, owner: &str, repo: &str, name: &str,
    ) -> Result<Arc<CachedFile>> {
        let release = self.latest_release(token, offline, owner, repo)?;

        let is_tarball = self.source_tarball && name == "tarball";
        let api_id = if is_tarball {
            release.api_id().unwrap_or(0)
        } else {
            release
                .asset_id(name)
                .ok_or_else(|| anyhow!("No such asset '{name}' in {owner}/{repo}"))?
        };

        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(a) = cache.get_asset(owner, repo, name, api_id) {
                return Ok(a);
            }
        }

        if offline {
            return Err(anyhow!("offline mode: no cached asset '{name}' for {owner}/{repo}"));
        }

        // Checked only after the cache tiers: an already-downloaded asset must stay
        // available even when the API is rate-limited.
        if self.rate_limited.load(Ordering::Relaxed) {
            return Err(anyhow!(RateLimitError { site: self.site }));
        }

        let url = if is_tarball {
            release
                .tarball_url()
                .ok_or_else(|| anyhow!("No tarball URL for {owner}/{repo}"))?
        } else {
            release
                .asset_download_url(name)
                .ok_or_else(|| anyhow!("No download URL for asset '{name}' in {owner}/{repo}"))?
        };

        if !url.starts_with("http:") && !url.starts_with("https:") {
            return Err(anyhow!("Unsafe URL scheme: {url}"));
        }

        log::info!("app={repo} msg=Downloading {name}");
        let mut req = ureq::get(&url).header("User-Agent", "relget");
        if self.auth_on_download
            && let Some(token) = token
        {
            let (hname, value) = (self.auth_header)(token);
            req = req.header(hname, &value);
        }
        let buf = req
            .call()
            .with_context(|| format!("Couldn't download '{}' from {}", name, self.site))?
            .into_body()
            .into_with_config()
            .limit(DOWNLOAD_LIMIT)
            .read_to_vec()
            .with_context(|| format!("Couldn't read downloaded asset '{name}'"))?;
        log::info!("app={repo} msg=Downloaded {name}");

        let asset = CachedFile {
            api_id,
            owner: owner.to_string(),
            repo: repo.to_string(),
            name: name.to_string(),
            data: buf,
        };
        self.cache.lock().unwrap().store_asset(asset)
    }
}
