use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::github::GithubClient;
use crate::types::{AppBinary, DownloadedAssets};
use crate::version::AppVersion;

use super::qsv::{OWNER, REPO, extract_named, gnu_zip_asset_name};

const NAMED_BINS: &[&str] =
    &["qsvdp", "qsvlite", "qsvmcp", "qsvp", "qsvpdp", "qsvplite", "qsvpmcp"];

pub struct QsvAll {
    client: Arc<GithubClient>,
}

impl QsvAll {
    pub const DESCRIPTION: &'static str =
        "High-performance CSV data-wrangling toolkit (all variants)";
    pub const URL: &'static str = "https://github.com/dathere/qsv";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for QsvAll {
    fn exe_name(&self) -> &str { "qsv" }

    fn released_version(&self) -> Result<AppVersion> {
        self.client.latest_release(OWNER, REPO)?.version()
    }

    fn download(&self) -> Result<DownloadedAssets> {
        let release = self.client.latest_release(OWNER, REPO)?;
        let name = gnu_zip_asset_name(&release)?;
        let asset = self.client.download_asset(OWNER, REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;

        let qsv_data = extract_named(&extractor, &members, "qsv")?;

        let mut other_bins = Vec::new();
        for &bin_name in NAMED_BINS {
            let data = extract_named(&extractor, &members, bin_name)?;
            other_bins.push(AppBinary::new(bin_name, data));
        }

        // qsvpy* glob
        for member in &members {
            let fname = match Path::new(member).file_name() {
                Some(f) => f.to_string_lossy().into_owned(),
                None => continue,
            };
            if fname.starts_with("qsvpy") && !fname.contains('.') {
                let data = extractor.extract(member)?;
                other_bins.push(AppBinary::new(fname, data));
            }
        }

        Ok(DownloadedAssets {
            binary: Some(AppBinary::new("qsv", qsv_data)),
            other_bins,
            ..Default::default()
        })
    }
}
