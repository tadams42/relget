use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::clients::GithubClient;
use crate::installer::{gen_completions_subcommand, run_cmd, with_temp_exe};
use crate::types::{AppBinary, Completion, AppAssets, ManPage};
use crate::version::AppVersion;

pub struct Caddy {
    client: Arc<GithubClient>,
}

impl Caddy {
    pub const ID: &'static str = "caddy";
    const OWNER: &'static str = "caddyserver";
    const REPO: &'static str = "caddy";
    const EXE_NAME: &'static str = "caddy";
    const VERSION_ARG: &'static str = "version";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Caddy {
    fn exe_name(&self) -> &str { Self::EXE_NAME }
    fn cli_version_arg(&self) -> &str { Self::VERSION_ARG }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary:      Some(AppBinary::descriptor("caddy")),
            man_pages:   vec![
                ManPage::descriptor(8, "caddy.8"),
                ManPage::descriptor(8, "caddy-adapt.8"),
                ManPage::descriptor(8, "caddy-add-package.8"),
                ManPage::descriptor(8, "caddy-build-info.8"),
                ManPage::descriptor(8, "caddy-completion.8"),
                ManPage::descriptor(8, "caddy-environ.8"),
                ManPage::descriptor(8, "caddy-file-server.8"),
                ManPage::descriptor(8, "caddy-file-server-export-template.8"),
                ManPage::descriptor(8, "caddy-fmt.8"),
                ManPage::descriptor(8, "caddy-hash-password.8"),
                ManPage::descriptor(8, "caddy-list-modules.8"),
                ManPage::descriptor(8, "caddy-manpage.8"),
                ManPage::descriptor(8, "caddy-reload.8"),
                ManPage::descriptor(8, "caddy-remove-package.8"),
                ManPage::descriptor(8, "caddy-respond.8"),
                ManPage::descriptor(8, "caddy-reverse-proxy.8"),
                ManPage::descriptor(8, "caddy-run.8"),
                ManPage::descriptor(8, "caddy-start.8"),
                ManPage::descriptor(8, "caddy-stop.8"),
                ManPage::descriptor(8, "caddy-storage.8"),
                ManPage::descriptor(8, "caddy-storage-export.8"),
                ManPage::descriptor(8, "caddy-storage-import.8"),
                ManPage::descriptor(8, "caddy-trust.8"),
                ManPage::descriptor(8, "caddy-untrust.8"),
                ManPage::descriptor(8, "caddy-upgrade.8"),
                ManPage::descriptor(8, "caddy-validate.8"),
                ManPage::descriptor(8, "caddy-version.8"),
            ],
            completions: vec![
                Completion::zsh_desc("caddy"),
                Completion::bash_desc("caddy"),
                Completion::fish_desc("caddy"),
            ],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release
            .asset_names()
            .into_iter()
            .find(|a| a.starts_with("caddy_") && a.ends_with("_linux_amd64.tar.gz"))
            .ok_or_else(|| anyhow!("Can't find caddy asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;
        let exe = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "caddy")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find caddy in archive"))?;
        let binary_data = extractor.extract(&exe)?;

        let (completions, man_pages) = with_temp_exe("caddy", &binary_data, |exe_path| {
            let completions = gen_completions_subcommand("caddy", &binary_data, "completion")?;

            // Generate man pages into a temp subdir
            let man_dir = exe_path.parent().unwrap().join("man");
            std::fs::create_dir_all(&man_dir)?;
            run_cmd(exe_path, &["manpage", "--directory", man_dir.to_str().unwrap()])?;

            let mut man_pages = Vec::new();
            for entry in std::fs::read_dir(&man_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("8") {
                    let fname = path.file_name().unwrap().to_str().unwrap().to_string();
                    let data = std::fs::read(&path)?;
                    man_pages.push(ManPage::new(8, fname, data));
                }
            }
            Ok((completions, man_pages))
        })?;

        Ok(AppAssets {
            binary: Some(AppBinary::new("caddy", binary_data)),
            completions,
            man_pages,
            ..Default::default()
        })
    }
}
