use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::version::AppVersion;

const RELEASE_CACHE_SECONDS: i64 = 86400;

// ── GhRelease ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhRelease {
    pub owner:         String,
    pub repo:          String,
    pub data:          Value,
    pub downloaded_at: DateTime<Utc>,
}

impl GhRelease {
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

    pub fn gh_id(&self) -> Option<u64> { self.data["id"].as_u64() }

    pub fn is_expired(&self) -> bool {
        Utc::now() - self.downloaded_at > Duration::seconds(RELEASE_CACHE_SECONDS)
    }
}

// ── GhDownloadedAsset ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct GhDownloadedAsset {
    pub gh_id: u64,
    pub owner: String,
    pub repo:  String,
    pub name:  String,
    pub data:  Vec<u8>,
}

// ── GhCache ──────────────────────────────────────────────────────────────────

pub struct GhCache {
    releases:  HashMap<String, GhRelease>,
    assets:    HashMap<String, GhDownloadedAsset>,
    cache_dir: PathBuf,
}

impl Default for GhCache {
    fn default() -> Self { Self::new() }
}

impl GhCache {
    pub fn new() -> Self { Self::new_with_prefix("") }

    pub fn new_with_prefix(subdir: &str) -> Self {
        let mut cache_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".cache")
            .join("relget");
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

    fn asset_key(gh_id: u64, name: &str) -> String {
        if name == "tarball" {
            format!("assets/tarball.{}", gh_id)
        } else {
            format!("assets/asset.{}", gh_id)
        }
    }

    fn repo_cache_dir(&self, owner: &str, repo: &str) -> PathBuf {
        self.cache_dir.join(owner).join(repo)
    }

    // ── releases ─────────────────────────────────────────────────────────────

    pub fn get_release(&mut self, owner: &str, repo: &str) -> Option<GhRelease> {
        let key = Self::release_key(owner, repo);

        // Check memory cache first
        if let Some(r) = self.releases.get(&key) {
            if !r.is_expired() {
                log::debug!("Memory cache hit for {}/{}", owner, repo);
                return Some(r.clone());
            }
            self.releases.remove(&key);
        }

        // Check disk cache
        let path = self.repo_cache_dir(owner, repo).join("release.json");
        if path.exists() {
            if let Ok(data) = std::fs::read_to_string(&path) {
                if let Ok(json) = serde_json::from_str::<GhRelease>(&data) {
                    if !json.is_expired() {
                        log::debug!("Disk cache hit for {}/{}", owner, repo);
                        self.releases.insert(key, json.clone());
                        return Some(json);
                    }
                }
            }
        }

        None
    }

    /// Like `get_release` but ignores expiry — for offline mode.
    pub fn get_release_any_age(&mut self, owner: &str, repo: &str) -> Option<GhRelease> {
        let key = Self::release_key(owner, repo);
        if let Some(r) = self.releases.get(&key) {
            return Some(r.clone());
        }
        let path = self.repo_cache_dir(owner, repo).join("release.json");
        if path.exists() {
            if let Ok(data) = std::fs::read_to_string(&path) {
                if let Ok(json) = serde_json::from_str::<GhRelease>(&data) {
                    self.releases.insert(key, json.clone());
                    return Some(json);
                }
            }
        }
        None
    }

    pub fn store_release(&mut self, release: GhRelease) -> Result<()> {
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
        &mut self, owner: &str, repo: &str, name: &str, gh_id: u64,
    ) -> Option<GhDownloadedAsset> {
        let key = Self::asset_key(gh_id, name);

        if let Some(a) = self.assets.get(&key) {
            log::debug!("Memory cache hit for asset {}", name);
            return Some(a.clone());
        }

        let file_name = if name == "tarball" {
            format!("tarball.{}", gh_id)
        } else {
            format!("asset.{}", gh_id)
        };
        let path = self.repo_cache_dir(owner, repo).join(&file_name);
        if path.exists() {
            if let Ok(data) = std::fs::read(&path) {
                log::debug!("Disk cache hit for asset {}", name);
                let asset = GhDownloadedAsset {
                    gh_id,
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

    pub fn store_asset(&mut self, asset: GhDownloadedAsset) -> Result<()> {
        let dir = self.repo_cache_dir(&asset.owner, &asset.repo);
        std::fs::create_dir_all(&dir)?;
        let file_name = if asset.name == "tarball" {
            format!("tarball.{}", asset.gh_id)
        } else {
            format!("asset.{}", asset.gh_id)
        };
        let path = dir.join(&file_name);
        std::fs::write(&path, &asset.data)?;
        let key = Self::asset_key(asset.gh_id, &asset.name);
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
