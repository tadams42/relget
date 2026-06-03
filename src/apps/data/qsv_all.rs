// Installs additional variants of `qsv` optimized for speciffic workloads. You probably
// don't need these for everyday work. Following are descriptions for all of them.
//
// ## Additional binaries
//
// ### Core Variants
//
// - `qsv`: The standard flagship version. It contains almost all built-in features (Luau scripting,
//   fast Polars dataframes, SQL support, geocoding, and web-fetching). It does not include the
//   Python engine to avoid requiring Python runtime libraries on your host system.
// - `qsvlite`: The stripped-down version. All heavy features (Polars, Luau, Python, Geocoding) are
//   stripped out, leaving only the ultra-fast core CSV processing commands (like count, slice,
//   split, sort). It is a tiny, highly portable binary with almost zero dependencies.
// - `qsvp`: The Polars-optimized version (The p stands for Polars/Parallel). This binary is
//   specifically optimized for memory-heavy or complex relational algebra operations like
//   vector-accelerated SQL queries, pivots, and joins.
//
// ### The Model Context Protocol (MCP) Variants
//
// - `qsvmcp`: A standard qsv engine stripped of non-essential features but embedding specialized
//   hooks to act smoothly as a local AI agent tool/backend.
// - `qsvplite`: A lightweight variant combining the p (Polars) base architecture but stripped down
//   (lite) for highly specific embedded compute constraints.
// - `qsvpmcp`: An MCP-optimized version built on top of the high-performance Polars (p) data
//   engine.
//
// ### Python Scripts (qsvpy*) Variants
//
// - `qsvpy*`: If you want to use the qsv py command to write custom Python scripts directly inside
//   your CSV pipelines, you must use one of these. Because Rust must link directly against a
//   specific Python shared library at compile time, you must pick the binary that exactly matches
//   the Python version installed on your Linux system:
//
// ### CKAN DataPusher Variants (*dp)
//
// The `dp` suffix stands for DataPusher+, an optimized pipeline engine designed to automatically
// ingest, clean, and pump massive tabular datasets directly into CKAN open-data portals.
// - `qsvdp`: The standard core binary optimized specifically for DataPusher open-data automation
//   environments.
// - `qsvpdp`: The Polars-accelerated variant optimized for heavy-lifting data data-pushing.
//
// ## Other files
//
// `qsv` downloads these when it needs them. But, it is advisable to set:
//
//     export QSV_CACHE_DIR="$HOME/.cache/qsv"
//
// - `qsv-20.1.0-geocode-index.rkyv.cities15000`: The uncompressed index database file built from
//   the Geonames Gazeteer database, filtered specifically for English-named cities around the globe
//   with a population greater than 15,000 (roughly ~26,000 cities). Serialized using `rkyv`, a
//   Rust-based zero-copy deserialization framework. It structures data so that its on-disk
//   footprint matches the in-memory representation, enabling `qsv` to map the file directly into
//   memory instantly without any processing overhead.
// - `qsv-20.1.0-geocode-index.rkyv`: This is actually the exact same file as the one above. If you
//   look closely at the release metadata, they share identical SHA-256 hashes. It is duplicated
//   under a shorter filename for automated fallback logic within qsv.
// - `qsv-20.1.0-geocode-index.rkyv.cities15000.sz`: The Snappy-compressed version of the database.
//   It is much smaller to download but requires decompression.
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppBinary, AppAssets};
use crate::version::AppVersion;

use super::qsv::{OWNER, REPO, extract_named, gnu_zip_asset_name};

const NAMED_BINS: &[&str] = &[
    "qsvdp", "qsvlite", "qsvmcp", "qsvp", "qsvpdp", "qsvplite", "qsvpmcp",
];

pub struct QsvAll {
    client: Arc<GithubClient>,
}

impl QsvAll {
    pub const ID: &'static str = "qsv-all";
    const EXE_NAME: &'static str = "qsv";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for QsvAll {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client.latest_release(OWNER, REPO)?.version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary:     Some(AppBinary::descriptor(Self::EXE_NAME)),
            other_bins: vec![
                AppBinary::descriptor("qsvdp"),
                AppBinary::descriptor("qsvlite"),
                AppBinary::descriptor("qsvmcp"),
                AppBinary::descriptor("qsvp"),
                AppBinary::descriptor("qsvpdp"),
                AppBinary::descriptor("qsvplite"),
                AppBinary::descriptor("qsvpmcp"),
                AppBinary::descriptor("qsvpy311"),
                AppBinary::descriptor("qsvpy312"),
                AppBinary::descriptor("qsvpy313"),
            ],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
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

        Ok(AppAssets {
            binary: Some(AppBinary::new("qsv", qsv_data)),
            other_bins,
            ..Default::default()
        })
    }
}
