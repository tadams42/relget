// Alternatives:
//
// [dust](https://github.com/bootandy/dust)
// - Rust
// - non-interactive static tree with horizontal bar charts
// - `dust` is built on the philosophy that you shouldn't need to pass a bunch of flags (like `-h`
//   for human-readable or `-d 1` for depth) to get a clear picture of your disk usage.
// - Instead of printing thousands of lines, it intelligently limits its output to roughly the
//   height of your terminal window, highlighting only the largest subdirectories and files.
// - It does an excellent job of showing exactly where a large file lives inside a deep folder
//   structure. In traditional tools, a folder might look large, but finding the exact culprit file
//   requires digging; dust bubbles the largest files right into the root tree printout.
//
// [pdu](https://github.com/KSXGitHub/parallel-disk-usage)
// - Rust, generates tree, but no colors
// - probably the fastest
// - pdu is heavily inspired by dust but aims to optimize raw performance and scriptability.
// - Visually, pdu looks very similar to dust, outputting a static tree with graphical horizontal
//   bars to depict size distributions.
// - One of pdu's standout features is its extreme friendliness to scripting. It includes
//   --json-output and --json-input flags. This allows you to scan a massive filesystem on a remote
//   server, export the layout to a lightweight JSON file, and then feed that JSON into pdu on a
//   local machine later to render the visualization.
// - It deliberately trades away certain edge-case features for raw speed; for instance, it does not
//   follow symbolic links, and it cannot naturally calculate reflinks on Copy-on-Write (COW)
//   filesystems like BTRFS or ZFS.
//
// [gdu](https://github.com/dundee/gdu)
// - Go TUI
// - a drop-in replacement for `ncdu` rewritten from scratch in Go to maximize multi-core execution.
// - Unlike dust and pdu, gdu drops you into a fully interactive terminal application. You use the
//   arrow keys (or Vim keys h/j/k/l) to dive into folders, sort entries by name or size, and press
//   the d key to delete space-wasting files safely right from the interface.
// - While it defaults to an interactive TUI, it will automatically fall back to a non-interactive
//   text summary if it detects it isn't running in a real terminal (e.g., if you pipe its output to
//   a file or a cron job log).
// - It includes specialized features like archive browsing (allowing you to look inside .zip,
//   .tar.gz, and .jar files without extracting them) and a constant-memory execution mode for
//   non-interactive scans.
//
// [ncdu](https://dev.yorhel.nl/ncdu)
// - `NCurses Disk Usage`
//
// [dutree](https://github.com/nachoparker/dutree)
// - Rust TUI
// - seems unmaintained
//
// [dua](https://github.com/Byron/dua-cli/)
// - Rust TUI
// - slightly unmaintained
//
// [dirstat-rs](https://github.com/scullionw/dirstat-rs)
// - Rust CLI that generates tree view
// - slightly unmaintained
//
// [godu](https://github.com/viktomas/godu)
// - Go TUI
// - for finding stuff that occupies most of disk space
//
// [tdu](https://github.com/josephpaul0/tdu)
// - prints output similar to `top`
// - seems unmaintained
//
// [duc](https://github.com/zevv/duc)
// - C, doesn't have pre-compiled static binaries
// - collection of tools for indexing, inspecting and visualizing disk usage
// - most scalable: "it has been tested on systems with more than 500 million files and several
//   petabytes of storage"
use std::path::Path;
use std::sync::Arc;

use anyhow::{Result, anyhow};

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::types::{AppBinary, Completion, AppAssets, ManPage};
use crate::version::AppVersion;

pub struct Dust {
    client: Arc<GithubClient>,
}

impl Dust {
    pub const ID: &'static str = "dust";
    pub const DESCRIPTION: &'static str = "Intuitive disk usage visualizer, a du alternative";
    pub const URL: &'static str = "https://github.com/bootandy/dust";
    const OWNER: &'static str = "bootandy";
    const REPO: &'static str = "dust";
    const EXE_NAME: &'static str = "dust";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Dust {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary:      Some(AppBinary::descriptor("dust")),
            man_pages:   vec![ManPage::descriptor(1, "dust.1")],
            completions: vec![Completion::zsh_desc("dust"), Completion::bash_desc("dust"), Completion::fish_desc("dust")],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;

        // Binary: x86_64 musl static build
        let bin_name = release
            .asset_names()
            .into_iter()
            .find(|a| a.contains("x86_64-unknown-linux-musl") && a.ends_with(".tar.gz"))
            .ok_or_else(|| anyhow!("Can't find dust musl binary asset"))?;
        let asset = self
            .client
            .download_asset(Self::OWNER, Self::REPO, &bin_name)?;
        let extractor = ArchiveExtractor::new(&bin_name, asset.data);
        let members = extractor.members()?;
        let exe = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "dust")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find dust executable in archive"))?;
        let binary_data = extractor.extract(&exe)?;

        // Man pages + completions: from the amd64 deb
        let deb_name = release
            .asset_names()
            .into_iter()
            .find(|a| a.starts_with("du-dust_") && a.ends_with("_amd64.deb"))
            .ok_or_else(|| anyhow!("Can't find du-dust deb asset"))?;
        let deb_asset = self
            .client
            .download_asset(Self::OWNER, Self::REPO, &deb_name)?;
        let deb_extractor = ArchiveExtractor::new(&deb_name, deb_asset.data);

        // deb is an ar archive; find the data.tar.* member (gz/xz/zst)
        let deb_members = deb_extractor.members()?;
        let data_tar_name = deb_members
            .iter()
            .find(|m| m.starts_with("data.tar"))
            .cloned()
            .ok_or_else(|| anyhow!("Can't find data.tar in deb"))?;
        let data_tar_data = deb_extractor.extract(&data_tar_name)?;

        let data_extractor = ArchiveExtractor::new(&data_tar_name, data_tar_data);
        let data_members = data_extractor.members()?;

        // Man pages — may be stored as .N.gz inside the tar
        let mut man_pages = Vec::new();
        for member in &data_members {
            if !member.contains("/man/") {
                continue;
            }
            let path = Path::new(member);
            let file_name = path.file_name().and_then(|f| f.to_str()).unwrap_or("");

            if let Some(stem) = file_name.strip_suffix(".gz") {
                // e.g. dust.1.gz → decompress, install as dust.1
                if let Some(Ok(section)) = Path::new(stem)
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.parse::<u8>())
                {
                    let compressed = data_extractor.extract(member)?;
                    let decompressed =
                        ArchiveExtractor::new("man.gz", compressed).extract("man")?;
                    man_pages.push(ManPage::new(section, stem, decompressed));
                }
            } else if let Some(Ok(section)) = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.parse::<u8>())
            {
                let data = data_extractor.extract(member)?;
                man_pages.push(ManPage::new(section, file_name, data));
            }
        }

        // Completions
        let mut completions = Vec::new();
        if let Some(m) = data_members.iter().find(|m| {
            m.contains("bash-completion")
                && Path::new(m)
                    .file_name()
                    .map(|f| f == "dust")
                    .unwrap_or(false)
        }) {
            completions.push(Completion::bash("dust", data_extractor.extract(m)?));
        }
        if let Some(m) = data_members.iter().find(|m| {
            Path::new(m)
                .file_name()
                .map(|f| f == "_dust")
                .unwrap_or(false)
        }) {
            completions.push(Completion::zsh("dust", data_extractor.extract(m)?));
        }
        if let Some(m) = data_members.iter().find(|m| {
            Path::new(m)
                .file_name()
                .map(|f| f == "dust.fish")
                .unwrap_or(false)
        }) {
            completions.push(Completion::fish("dust", data_extractor.extract(m)?));
        }

        Ok(AppAssets {
            binary: Some(AppBinary::new("dust", binary_data)),
            man_pages,
            completions,
            ..Default::default()
        })
    }
}
