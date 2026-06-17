// gocryptfs (Recommended for general use): It is fast, security-audited, actively
// maintained, and handles local/cloud synchronization exceptionally well. It is the
// modern spiritual successor to EncFS.
// - Performance: It is incredibly lightweight. It takes full advantage of CPU hardware acceleration
//   (AES-NI), hitting high read/write throughput that matches or pushes close to native disk
//   speeds.
// - Security: Successfully passed a rigorous professional third-party audit. It secures both file
//   data and handles directory/filename obfuscation cleanly (using an initialization vector per
//   directory to prevent identical filenames from looking identical when encrypted).
// - Leaks file sizes and the overall count/structure of files to anyone monitoring the raw storage
//   folder.
//
// [CryFS](https://github.com/cryfs) (Recommended for maximum privacy): It provides
// superior security by masking metadata (file sizes and directory structures), but it
// introduces a major performance trade-off and increased storage overhead.
// - CryFS was created specifically to solve the metadata leakage inherent to 1:1 overlay
//   filesystems.
// - Unrivaled Privacy: It provides the closest thing to full-disk encryption security inside a
//   cloud-synchronized folder. It completely masks file counts, individual sizes, and directory
//   topology. All data is encrypted and represented on disk as blocks of the roughly same size
// - Performance Hit: The layer of abstraction required to shred files into uniform blocks results
//   in noticeably slower read/write operations and higher CPU utilization, especially with deep
//   directory trees or small file alterations.
// - Sync Friction: Because it updates base blocks and pointers, synchronization utilities can
//   occasionally suffer from race conditions or conflict files if data is altered simultaneously on
//   multiple machines before a sync cycle completes.
//
// EncFS (Not Recommended): It is considered legacy software. Due to fundamental
// cryptographic flaws discovered in audits, it is unsafe to use, particularly in
// untrusted cloud environments where attackers can access historical snapshots of
// files.

use anyhow::Result;
use std::sync::Arc;

use crate::apps::App;
use crate::apps::app_assets::{AppAssets, AppBinary, ManPage};
use crate::archive::ArchiveExtractor;
use crate::clients::RelgetClient;
use crate::version::AppVersion;

pub struct Gocryptfs {
    client: Arc<dyn RelgetClient>,
}

impl Gocryptfs {
    pub const ID: &'static str = "gocryptfs";
    const OWNER: &'static str = "rfjakob";
    const REPO: &'static str = "gocryptfs";
    const EXE_NAME: &'static str = "gocryptfs";

    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Gocryptfs {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::new(Self::EXE_NAME)),
            other_bins: vec![AppBinary::new("gocryptfs-xray")],
            man_pages: vec![
                ManPage::new(1, "gocryptfs.1"),
                ManPage::new(1, "gocryptfs-xray.1"),
            ],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release.find_asset(|a| a.ends_with("_linux-static_amd64.tar.gz"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data(
                Self::EXE_NAME,
                extractor.extract_by_filename(Self::EXE_NAME)?,
            )),
            other_bins: vec![AppBinary::new_with_data(
                "gocryptfs-xray",
                extractor.extract_by_filename("gocryptfs-xray")?,
            )],
            man_pages: vec![
                ManPage::new_with_data(
                    1,
                    "gocryptfs.1",
                    extractor.extract_by_filename("gocryptfs.1")?,
                ),
                ManPage::new_with_data(
                    1,
                    "gocryptfs-xray.1",
                    extractor.extract_by_filename("gocryptfs-xray.1")?,
                ),
            ],
            ..Default::default()
        })
    }
}
