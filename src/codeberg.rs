use anyhow::{anyhow, Context, Result};
use once_cell::sync::Lazy;
use std::sync::Mutex;

use crate::cache::{GhCache, GhDownloadedAsset, GhRelease};

static CACHE: Lazy<Mutex<GhCache>> = Lazy::new(|| Mutex::new(GhCache::new_with_prefix("codeberg")));

const CB_API_URL: &str = "https://codeberg.org/api/v1/repos";

pub struct CodebergClient {
    pub token: Option<String>,
}

impl CodebergClient {
    pub fn new(token: Option<String>) -> Self {
        Self { token: token.or_else(|| Self::load_token()) }
    }

    fn load_token() -> Option<String> {
        if let Ok(t) = std::env::var("CODEBERG_API_TOKEN") {
            if !t.is_empty() {
                return Some(t);
            }
        }
        let config_path = dirs::home_dir()
            .unwrap_or_default()
            .join(".config")
            .join("codeberg")
            .join("api_token");
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                let token = content.lines().last().unwrap_or("").trim().to_string();
                if !token.is_empty() {
                    return Some(token);
                }
            }
        }
        None
    }

    pub fn latest_release(&self, owner: &str, repo: &str) -> Result<GhRelease> {
        {
            let mut cache = CACHE.lock().unwrap();
            if let Some(r) = cache.get_release(owner, repo) {
                return Ok(r);
            }
        }

        log::info!("app={} msg=Fetching latest Codeberg release", repo);
        let url = format!("{}/{}/{}/releases?limit=5&page=1", CB_API_URL, owner, repo);

        let mut req = ureq::get(&url)
            .header("Accept", "application/json")
            .header("User-Agent", "binup");
        if let Some(token) = &self.token {
            req = req.header("Authorization", &format!("token {}", token));
        }
        let releases: Vec<serde_json::Value> = req
            .call()
            .with_context(|| format!("Can't fetch Codeberg release info for {}/{}", owner, repo))?
            .into_body()
            .read_json()
            .with_context(|| format!("Invalid JSON from Codeberg for {}/{}", owner, repo))?;

        let data = releases
            .into_iter()
            .find(|r| {
                r["assets"].as_array().map(|a| !a.is_empty()).unwrap_or(false)
                    && !r["draft"].as_bool().unwrap_or(false)
                    && !r["prerelease"].as_bool().unwrap_or(false)
            })
            .ok_or_else(|| anyhow!("No release with assets for {}/{}", owner, repo))?;

        let release = GhRelease::new(owner, repo, data)?;
        CACHE.lock().unwrap().store_release(release.clone())?;
        Ok(release)
    }

    pub fn download_asset(&self, owner: &str, repo: &str, name: &str) -> Result<GhDownloadedAsset> {
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

        let url = release
            .asset_download_url(name)
            .ok_or_else(|| anyhow!("No download URL for asset '{}' in {}/{}", name, owner, repo))?;

        if !url.starts_with("http:") && !url.starts_with("https:") {
            return Err(anyhow!("Unsafe URL scheme: {}", url));
        }

        log::info!("app={} msg=Downloading {}", repo, name);
        let mut req = ureq::get(&url).header("User-Agent", "binup");
        if let Some(token) = &self.token {
            req = req.header("Authorization", &format!("token {}", token));
        }
        let buf = req
            .call()
            .with_context(|| format!("Couldn't download '{}' from Codeberg", name))?
            .into_body()
            .read_to_vec()
            .with_context(|| format!("Couldn't read downloaded asset '{}'", name))?;
        log::info!("app={} msg=Downloaded {}", repo, name);

        let asset = GhDownloadedAsset {
            gh_id: asset_id,
            owner: owner.to_string(),
            repo: repo.to_string(),
            name: name.to_string(),
            data: buf,
        };
        CACHE.lock().unwrap().store_asset(asset.clone())?;
        Ok(asset)
    }
}
