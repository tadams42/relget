use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::version::AppVersion;

const RELEASE_CACHE_SECONDS: i64 = 86400;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseMetadata {
    pub owner:         String,
    pub repo:          String,
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

    pub fn find_asset(&self, predicate: impl Fn(&str) -> bool) -> Result<String> {
        self.asset_names()
            .into_iter()
            .find(|a| predicate(a.as_str()))
            .ok_or_else(|| anyhow!("No matching asset found in {}/{}", self.owner, self.repo))
    }

    pub fn asset_download_url(&self, name: &str) -> Option<String> {
        self.data["assets"].as_array()?.iter().find_map(|v| {
            if v["name"].as_str() == Some(name) {
                v["browser_download_url"].as_str().map(|s| s.to_string())
            } else {
                None
            }
        })
    }

    pub fn asset_id(&self, name: &str) -> Option<u64> {
        self.data["assets"].as_array()?.iter().find_map(|v| {
            if v["name"].as_str() == Some(name) {
                v["id"].as_u64()
            } else {
                None
            }
        })
    }

    pub fn tarball_url(&self) -> Option<String> {
        self.data["tarball_url"].as_str().map(|s| s.to_string())
    }

    pub fn api_id(&self) -> Option<u64> { self.data["id"].as_u64() }

    pub fn is_expired(&self) -> bool {
        Utc::now() - self.downloaded_at > Duration::seconds(RELEASE_CACHE_SECONDS)
    }
}

#[derive(Debug, Clone)]
pub struct CachedFile {
    pub api_id: u64,
    pub owner:  String,
    pub repo:   String,
    pub name:   String,
    pub data:   Vec<u8>,
}

pub struct RelgetCache {
    releases:  HashMap<String, ReleaseMetadata>,
    assets:    HashMap<String, CachedFile>,
    cache_dir: PathBuf,
}

impl Default for RelgetCache {
    fn default() -> Self { Self::new() }
}

impl RelgetCache {
    pub fn new() -> Self { Self::new_with_prefix("") }

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

    fn release_key(owner: &str, repo: &str) -> String { format!("releases/{}/{}", owner, repo) }

    fn asset_key(api_id: u64, name: &str) -> String {
        if name == "tarball" {
            format!("assets/tarball.{}", api_id)
        } else {
            format!("assets/asset.{}", api_id)
        }
    }

    fn repo_cache_dir(&self, owner: &str, repo: &str) -> PathBuf {
        self.cache_dir.join(owner).join(repo)
    }

    // ── releases ─────────────────────────────────────────────────────────────

    pub fn get_release(&mut self, owner: &str, repo: &str) -> Option<ReleaseMetadata> {
        let key = Self::release_key(owner, repo);

        // Check memory cache first
        if let Some(r) = self.releases.get(&key) {
            if !r.is_expired() {
                log::debug!("owner={} repo={} msg=memory-cache-hit", owner, repo);
                return Some(r.clone());
            }
            self.releases.remove(&key);
        }

        // Check disk cache
        let path = self.repo_cache_dir(owner, repo).join("release.json");
        if path.exists() {
            if let Ok(data) = std::fs::read_to_string(&path) {
                if let Ok(json) = serde_json::from_str::<ReleaseMetadata>(&data) {
                    if !json.is_expired() {
                        log::debug!("owner={} repo={} msg=disk-cache-hit", owner, repo);
                        self.releases.insert(key, json.clone());
                        return Some(json);
                    }
                }
            }
        }

        None
    }

    /// Like `get_release` but ignores expiry — for offline mode.
    pub fn get_release_any_age(&mut self, owner: &str, repo: &str) -> Option<ReleaseMetadata> {
        let key = Self::release_key(owner, repo);
        if let Some(r) = self.releases.get(&key) {
            return Some(r.clone());
        }
        let path = self.repo_cache_dir(owner, repo).join("release.json");
        if path.exists() {
            if let Ok(data) = std::fs::read_to_string(&path) {
                if let Ok(json) = serde_json::from_str::<ReleaseMetadata>(&data) {
                    self.releases.insert(key, json.clone());
                    return Some(json);
                }
            }
        }
        None
    }

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

    pub fn get_asset(
        &mut self, owner: &str, repo: &str, name: &str, api_id: u64,
    ) -> Option<CachedFile> {
        let key = Self::asset_key(api_id, name);

        if let Some(a) = self.assets.get(&key) {
            log::debug!("asset={} msg=memory-cache-hit", name);
            return Some(a.clone());
        }

        let file_name = if name == "tarball" {
            format!("tarball.{}", api_id)
        } else {
            format!("asset.{}", api_id)
        };
        let path = self.repo_cache_dir(owner, repo).join(&file_name);
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

    pub fn store_asset(&mut self, asset: CachedFile) -> Result<()> {
        let dir = self.repo_cache_dir(&asset.owner, &asset.repo);
        std::fs::create_dir_all(&dir)?;
        let file_name = if asset.name == "tarball" {
            format!("tarball.{}", asset.api_id)
        } else {
            format!("asset.{}", asset.api_id)
        };
        let path = dir.join(&file_name);
        std::fs::write(&path, &asset.data)?;
        let key = Self::asset_key(asset.api_id, &asset.name);
        self.assets.insert(key, asset);
        Ok(())
    }
}

// ── version extraction ───────────────────────────────────────────────────────

fn extract_version(data: &Value) -> Option<AppVersion> {
    // Try tag_name, then name, then body
    for field in &["tag_name", "name"] {
        if let Some(s) = data[field].as_str() {
            if let Some(v) = AppVersion::parse(s) {
                return Some(v);
            }
        }
    }

    // Try body text — look for x.y.z pattern
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
