//! Two-tier (memory + disk) cache for release metadata and binary assets.
//!
//! # Architecture
//!
//! [`RelgetCache`] maintains two in-memory [`HashMap`]s — one for release
//! metadata, one for binary assets — and backs both with files under
//! `~/.cache/relget/`.  This avoids redundant HTTP API calls across invocations
//! (disk tier) and redundant disk reads within a single invocation (memory tier).
//!
//! # Provider isolation
//!
//! Each provider (GitHub, Codeberg, GitLab) creates its own [`RelgetCache`]
//! instance via [`RelgetCache::new`] (GitHub) or [`RelgetCache::new_with_prefix`]
//! (Codeberg / GitLab).  In practice each provider module wraps its instance in a
//! `Lazy<Mutex<RelgetCache>>` singleton so the cache persists across calls within
//! one process.
//!
//! On-disk layout:
//!
//! ```text
//! ~/.cache/relget/
//!   {owner}/{repo}/release.json             # GitHub release metadata
//!   {owner}/{repo}/asset.{id}               # GitHub release asset
//!   {owner}/{repo}/tarball.{id}             # GitHub source tarball
//!   codeberg/{owner}/{repo}/release.json    # Codeberg
//!   gitlab/{owner}/{repo}/release.json      # GitLab
//! ```
//!
//! # Normalised JSON contract
//!
//! [`ReleaseMetadata::data`] always carries **GitHub-API-shaped** JSON regardless
//! of which provider was queried.  Codeberg and GitLab normalise their API
//! responses into this shape *before* calling [`RelgetCache::store_release`]; the
//! normalisation lives in the respective client files, not here.  That is why
//! every accessor on [`ReleaseMetadata`] works uniformly across all providers.
//!
//! # Caching policies
//!
//! | Kind    | On-disk key | TTL |
//! |---------|-------------|-----|
//! | Release | `{owner}/{repo}/release.json` | 1 day — expired entries are evicted on read and re-fetched from the API |
//! | Asset   | `{owner}/{repo}/asset.{id}` | Permanent — keyed by API asset ID, which changes when a new release is published, so stale entries are superseded naturally |
//! | Tarball | `{owner}/{repo}/tarball.{id}` | Same permanent policy as assets |
//!
//! The string `"tarball"` is a reserved sentinel passed as the `name` argument to
//! [`RelgetCache::get_asset`] / [`RelgetCache::store_asset`] to request the
//! source-code tarball (`tarball_url` in the GitHub API).  It gets its own
//! `tarball.{id}` filename prefix on disk so it never collides with a release
//! asset that happens to be named `"tarball"`.
//!
//! # Offline mode
//!
//! [`RelgetCache::get_release_any_age`] is identical to [`RelgetCache::get_release`]
//! but skips the TTL check.  It is called by the provider client modules when the
//! `--offline` flag is active, so those callers never need to know the expiry
//! policy themselves.
//!
//! # Write-through semantics
//!
//! [`RelgetCache::store_release`] and [`RelgetCache::store_asset`] always write to
//! disk first and then insert into the in-memory map.  A crash between the two
//! steps leaves the disk as the authoritative store.
//!
//! # Typical caller pattern
//!
//! ```rust,ignore
//! // Inside a provider client (e.g. github.rs):
//! let mut cache = CACHE.lock().unwrap();
//!
//! if let Some(release) = cache.get_release(owner, repo) {
//!     return Ok(release); // served from memory or disk
//! }
//!
//! // Cache miss — fetch fresh data and write through.
//! let fresh = fetch_from_api(owner, repo)?;
//! cache.store_release(fresh.clone())?;
//! Ok(fresh)
//! ```

use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::version::AppVersion;

const RELEASE_CACHE_SECONDS: i64 = 86400;

/// Release metadata fetched from a forge API and stored in the cache.
///
/// `data` always holds **GitHub-API-shaped** JSON (see the [module-level
/// normalisation contract](self#normalised-json-contract)).  The accessor
/// methods on this type — [`asset_names`], [`asset_download_url`], etc. —
/// decode fields from that shape.
///
/// `downloaded_at` records when the data was fetched.  [`is_expired`] compares
/// it against the 1-day TTL to decide whether the entry needs to be re-fetched.
///
/// [`asset_names`]: Self::asset_names
/// [`asset_download_url`]: Self::asset_download_url
/// [`is_expired`]: Self::is_expired
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseMetadata {
    pub owner:         String,
    pub repo:          String,
    /// GitHub-API-shaped release JSON (normalised for Codeberg / GitLab).
    pub data:          Value,
    pub downloaded_at: DateTime<Utc>,
}

impl ReleaseMetadata {
    pub fn new(owner: &str, repo: &str, data: Value) -> Result<Self> {
        Ok(Self {
            owner: owner.to_string(),
            repo: repo.to_string(),
            data,
            downloaded_at: Utc::now(),
        })
    }

    pub fn version(&self) -> Result<AppVersion> {
        extract_version(&self.data)
            .with_context(|| format!("No version info in {}/{} release", self.owner, self.repo))
    }

    /// Names of all release assets (`assets[].name` in the JSON).
    pub fn asset_names(&self) -> Vec<String> {
        self.data["assets"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v["name"].as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Returns the name of the first asset matching `predicate`, or an error.
    pub fn find_asset(&self, predicate: impl Fn(&str) -> bool) -> Result<String> {
        self.asset_names()
            .into_iter()
            .find(|a| predicate(a.as_str()))
            .ok_or_else(|| anyhow!("No matching asset found in {}/{}", self.owner, self.repo))
    }

    /// Download URL (`browser_download_url`) for the named asset, if present.
    pub fn asset_download_url(&self, name: &str) -> Option<String> {
        self.data["assets"].as_array()?.iter().find_map(|v| {
            if v["name"].as_str() == Some(name) {
                v["browser_download_url"].as_str().map(|s| s.to_string())
            } else {
                None
            }
        })
    }

    /// Numeric API ID of the named asset (`assets[].id`), used as a cache key.
    pub fn asset_id(&self, name: &str) -> Option<u64> {
        self.data["assets"].as_array()?.iter().find_map(|v| {
            if v["name"].as_str() == Some(name) {
                v["id"].as_u64()
            } else {
                None
            }
        })
    }

    /// URL of the source-code tarball (`tarball_url` field), if present.
    pub fn tarball_url(&self) -> Option<String> {
        self.data["tarball_url"].as_str().map(|s| s.to_string())
    }

    /// Numeric API ID of the release itself (`id` field), used as the tarball cache key.
    pub fn api_id(&self) -> Option<u64> { self.data["id"].as_u64() }

    /// Returns `true` if the entry is older than the 1-day release cache TTL.
    pub fn is_expired(&self) -> bool {
        Utc::now() - self.downloaded_at > Duration::seconds(RELEASE_CACHE_SECONDS)
    }
}

/// A downloaded binary asset or source tarball held in the cache.
///
/// Assets are cached permanently and keyed by the API-assigned numeric ID rather
/// than by filename or version.  When a new release is published the IDs change,
/// so stale entries are naturally superseded without explicit invalidation.
///
/// `name` uses the special sentinel value `"tarball"` for source-code tarballs;
/// any other value is a named release asset.  This distinction controls the
/// on-disk filename prefix — see [`disk_filename`].
#[derive(Debug, Clone)]
pub struct CachedFile {
    pub api_id: u64,
    pub owner:  String,
    pub repo:   String,
    pub name:   String,
    pub data:   Vec<u8>,
}

/// Two-tier (memory + disk) cache for release metadata and binary assets.
///
/// Create one instance per provider:
/// - [`RelgetCache::new`] for GitHub (root cache directory)
/// - [`RelgetCache::new_with_prefix("codeberg")`](Self::new_with_prefix) for Codeberg
/// - [`RelgetCache::new_with_prefix("gitlab")`](Self::new_with_prefix) for GitLab
///
/// Each provider module wraps its instance in a `Lazy<Mutex<RelgetCache>>`
/// singleton so the cache persists for the lifetime of the process.
///
/// See the [module-level documentation](self) for the full architecture,
/// caching policies, and typical caller patterns.
pub struct RelgetCache {
    releases:  HashMap<String, ReleaseMetadata>,
    assets:    HashMap<String, CachedFile>,
    cache_dir: PathBuf,
}

impl Default for RelgetCache {
    fn default() -> Self { Self::new() }
}

impl RelgetCache {
    /// Creates a cache rooted at `~/.cache/relget/` (used for GitHub).
    pub fn new() -> Self { Self::new_with_prefix("") }

    /// Creates a cache rooted at `~/.cache/relget/{subdir}/`.
    ///
    /// Used by Codeberg (`"codeberg"`) and GitLab (`"gitlab"`) to keep their
    /// files isolated from GitHub's and from each other.
    pub fn new_with_prefix(subdir: &str) -> Self {
        let mut cache_dir = xdg::BaseDirectories::with_prefix("relget")
            .get_cache_home()
            .unwrap_or_else(|| PathBuf::from(".cache/relget"));
        if !subdir.is_empty() {
            cache_dir = cache_dir.join(subdir);
        }
        Self {
            releases: HashMap::new(),
            assets: HashMap::new(),
            cache_dir,
        }
    }

    /// In-memory and on-disk lookup key for a release: `"releases/{owner}/{repo}"`.
    fn release_key(owner: &str, repo: &str) -> String { format!("releases/{}/{}", owner, repo) }

    /// In-memory lookup key for an asset: `"assets/asset.{id}"` for release
    /// assets, `"assets/tarball.{id}"` for source tarballs.
    fn asset_key(api_id: u64, name: &str) -> String {
        format!("assets/{}", disk_filename(name, api_id))
    }

    fn repo_cache_dir(&self, owner: &str, repo: &str) -> PathBuf {
        self.cache_dir.join(owner).join(repo)
    }

    // ── releases ─────────────────────────────────────────────────────────────

    /// Returns a non-expired release from the memory or disk cache, promoting a
    /// disk hit to the memory tier.
    ///
    /// Returns `None` if no entry exists or if the cached entry has exceeded the
    /// 1-day TTL.  In both cases the caller is expected to fetch fresh data from
    /// the API and call [`store_release`].  Stale memory entries are evicted
    /// before the disk is checked.
    ///
    /// For offline use (TTL ignored) see [`get_release_any_age`].
    ///
    /// [`store_release`]: Self::store_release
    /// [`get_release_any_age`]: Self::get_release_any_age
    pub fn get_release(&mut self, owner: &str, repo: &str) -> Option<ReleaseMetadata> {
        let key = Self::release_key(owner, repo);

        if let Some(r) = self.releases.get(&key) {
            if !r.is_expired() {
                log::debug!("owner={} repo={} msg=memory-cache-hit", owner, repo);
                return Some(r.clone());
            }
            self.releases.remove(&key);
        }

        let path = self.repo_cache_dir(owner, repo).join("release.json");
        if let Some(json) = load_release_from_disk(&path) {
            if !json.is_expired() {
                log::debug!("owner={} repo={} msg=disk-cache-hit", owner, repo);
                self.releases.insert(key, json.clone());
                return Some(json);
            }
        }

        None
    }

    /// Like [`get_release`] but ignores the TTL — for `--offline` mode.
    ///
    /// Returns whatever is in the cache (memory or disk), regardless of age.
    /// Returns `None` only if the release has never been cached at all.
    ///
    /// [`get_release`]: Self::get_release
    pub fn get_release_any_age(&mut self, owner: &str, repo: &str) -> Option<ReleaseMetadata> {
        let key = Self::release_key(owner, repo);
        if let Some(r) = self.releases.get(&key) {
            return Some(r.clone());
        }
        let path = self.repo_cache_dir(owner, repo).join("release.json");
        if let Some(json) = load_release_from_disk(&path) {
            self.releases.insert(key, json.clone());
            return Some(json);
        }
        None
    }

    /// Writes `release` to disk then inserts it into the memory cache (write-through).
    pub fn store_release(&mut self, release: ReleaseMetadata) -> Result<()> {
        let dir = self.repo_cache_dir(&release.owner, &release.repo);
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("release.json");
        let json = serde_json::to_string_pretty(&release)?;
        std::fs::write(&path, &json)?;
        let key = Self::release_key(&release.owner, &release.repo);
        self.releases.insert(key, release);
        Ok(())
    }

    // ── assets ───────────────────────────────────────────────────────────────

    /// Returns an asset from the memory or disk cache, promoting a disk hit to
    /// the memory tier.  Returns `None` if the asset has never been downloaded.
    ///
    /// Assets have no TTL; `api_id` encodes the release version (the ID changes
    /// when a new release is published), so a stale entry will simply never be
    /// requested again.
    ///
    /// Pass `name = "tarball"` to retrieve a source-code tarball; any other value
    /// is treated as a named release asset filename.
    pub fn get_asset(
        &mut self, owner: &str, repo: &str, name: &str, api_id: u64,
    ) -> Option<CachedFile> {
        let key = Self::asset_key(api_id, name);

        if let Some(a) = self.assets.get(&key) {
            log::debug!("asset={} msg=memory-cache-hit", name);
            return Some(a.clone());
        }

        let path = self
            .repo_cache_dir(owner, repo)
            .join(disk_filename(name, api_id));
        if path.exists() {
            if let Ok(data) = std::fs::read(&path) {
                log::debug!("asset={} msg=disk-cache-hit", name);
                let asset = CachedFile {
                    api_id,
                    owner: owner.to_string(),
                    repo: repo.to_string(),
                    name: name.to_string(),
                    data,
                };
                self.assets.insert(key, asset.clone());
                return Some(asset);
            }
        }

        None
    }

    /// Writes `asset` to disk then inserts it into the memory cache (write-through).
    ///
    /// Use `asset.name = "tarball"` for source-code tarballs; any other name is
    /// stored with an `"asset."` filename prefix.
    pub fn store_asset(&mut self, asset: CachedFile) -> Result<()> {
        let dir = self.repo_cache_dir(&asset.owner, &asset.repo);
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(disk_filename(&asset.name, asset.api_id));
        std::fs::write(&path, &asset.data)?;
        let key = Self::asset_key(asset.api_id, &asset.name);
        self.assets.insert(key, asset);
        Ok(())
    }
}

// ── private helpers ──────────────────────────────────────────────────────────

/// On-disk filename for an asset entry.
///
/// Returns `"tarball.{api_id}"` for source-code tarballs (sentinel name
/// `"tarball"`) and `"asset.{api_id}"` for all named release assets.
fn disk_filename(name: &str, api_id: u64) -> String {
    if name == "tarball" {
        format!("tarball.{}", api_id)
    } else {
        format!("asset.{}", api_id)
    }
}

/// Reads and deserialises a [`ReleaseMetadata`] from `path`.
///
/// Returns `None` if the file does not exist, is unreadable, or contains
/// invalid JSON — all treated as a cache miss.
fn load_release_from_disk(path: &Path) -> Option<ReleaseMetadata> {
    if !path.exists() {
        return None;
    }
    let data = std::fs::read_to_string(path).ok()?;
    serde_json::from_str::<ReleaseMetadata>(&data).ok()
}

// ── version extraction ───────────────────────────────────────────────────────

/// Attempts to parse a semver version from a release API response.
///
/// Tries fields in order: `tag_name` → `name` → first `x.y.z` pattern in
/// `body`.  Returns `None` if none of the fields yield a parseable version.
fn extract_version(data: &Value) -> Option<AppVersion> {
    for field in &["tag_name", "name"] {
        if let Some(s) = data[field].as_str() {
            if let Some(v) = AppVersion::parse(s) {
                return Some(v);
            }
        }
    }

    if let Some(body) = data["body"].as_str() {
        let re = regex::Regex::new(r"\d+\.\d+\.\d+").ok()?;
        for line in body.lines() {
            if let Some(m) = re.find(line) {
                if let Some(v) = AppVersion::parse(m.as_str()) {
                    return Some(v);
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use serde_json::json;

    fn metadata_from(data: Value) -> ReleaseMetadata {
        ReleaseMetadata {
            owner: "owner".to_string(),
            repo: "repo".to_string(),
            data,
            downloaded_at: Utc::now(),
        }
    }

    // ── asset_names ──────────────────────────────────────────────────────────

    #[test]
    fn asset_names_empty_when_key_missing() {
        let m = metadata_from(json!({}));
        assert!(m.asset_names().is_empty());
    }

    #[test]
    fn asset_names_returns_correct_names() {
        let m = metadata_from(json!({
            "assets": [
                {"name": "app-1.0.0-linux.tar.gz", "id": 1, "browser_download_url": "https://example.com/a"},
                {"name": "app-1.0.0-macos.tar.gz", "id": 2, "browser_download_url": "https://example.com/b"},
            ]
        }));
        assert_eq!(
            m.asset_names(),
            vec!["app-1.0.0-linux.tar.gz", "app-1.0.0-macos.tar.gz"]
        );
    }

    // ── find_asset ───────────────────────────────────────────────────────────

    #[test]
    fn find_asset_matches_by_predicate() {
        let m = metadata_from(json!({
            "assets": [
                {"name": "app-linux.tar.gz"},
                {"name": "app-macos.tar.gz"},
            ]
        }));
        let name = m.find_asset(|n| n.contains("linux")).unwrap();
        assert_eq!(name, "app-linux.tar.gz");
    }

    #[test]
    fn find_asset_returns_err_when_no_match() {
        let m = metadata_from(json!({"assets": [{"name": "app-windows.zip"}]}));
        assert!(m.find_asset(|n| n.contains("linux")).is_err());
    }

    // ── asset_download_url ───────────────────────────────────────────────────

    #[test]
    fn asset_download_url_returns_some_for_known_name() {
        let m = metadata_from(json!({
            "assets": [{"name": "app.tar.gz", "browser_download_url": "https://example.com/app.tar.gz"}]
        }));
        assert_eq!(
            m.asset_download_url("app.tar.gz"),
            Some("https://example.com/app.tar.gz".to_string())
        );
    }

    #[test]
    fn asset_download_url_returns_none_for_unknown_name() {
        let m = metadata_from(
            json!({"assets": [{"name": "other.tar.gz", "browser_download_url": "https://x"}]}),
        );
        assert!(m.asset_download_url("app.tar.gz").is_none());
    }

    // ── version ──────────────────────────────────────────────────────────────

    #[test]
    fn version_extracted_from_tag_name() {
        let m = metadata_from(json!({"tag_name": "v1.2.3"}));
        assert_eq!(m.version().unwrap(), AppVersion(1, 2, 3));
    }

    #[test]
    fn version_falls_back_to_name_field() {
        let m = metadata_from(json!({"name": "Release 2.0.0"}));
        assert_eq!(m.version().unwrap(), AppVersion(2, 0, 0));
    }

    #[test]
    fn version_returns_err_when_no_version_fields() {
        let m = metadata_from(json!({"description": "no version here"}));
        assert!(m.version().is_err());
    }

    // ── is_expired ───────────────────────────────────────────────────────────

    #[test]
    fn is_expired_false_for_fresh_metadata() {
        let m = ReleaseMetadata::new("o", "r", json!({})).unwrap();
        assert!(!m.is_expired());
    }

    #[test]
    fn is_expired_true_for_old_metadata() {
        let m = ReleaseMetadata {
            owner:         "o".to_string(),
            repo:          "r".to_string(),
            data:          json!({}),
            downloaded_at: Utc::now() - Duration::seconds(86401),
        };
        assert!(m.is_expired());
    }
}
