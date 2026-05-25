use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;

use crate::apps::App;
use crate::archive::ArchiveExtractor;
use crate::github::GithubClient;
use crate::installer::{gen_completions_subcommand, run_cmd, with_temp_exe};
use crate::types::{AppBinary, DownloadedAssets, ManPage};
use crate::version::AppVersion;

pub struct Dasel {
    client: Arc<GithubClient>,
}

impl Dasel {
    pub const ID: &'static str = "dasel";
    pub const DESCRIPTION: &'static str = "Query and modify data in JSON, YAML, TOML, XML, and CSV";
    pub const URL: &'static str = "https://github.com/TomWright/dasel";
    const OWNER: &'static str = "TomWright";
    const REPO: &'static str = "dasel";
    const EXE_NAME: &'static str = "dasel";
    const VERSION_ARG: &'static str = "version";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for Dasel {
    fn exe_name(&self) -> &str { Self::EXE_NAME }
    fn cli_version_arg(&self) -> &str { Self::VERSION_ARG }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn download(&self) -> Result<DownloadedAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release
            .asset_names()
            .into_iter()
            .find(|a| a == "dasel_linux_amd64.gz")
            .ok_or_else(|| anyhow!("Can't find dasel_linux_amd64.gz"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let extractor = ArchiveExtractor::new(&name, asset.data);
        let members = extractor.members()?;
        let exe_member = members
            .iter()
            .find(|m| {
                Path::new(m)
                    .file_name()
                    .map(|f| f == "dasel_linux_amd64")
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find dasel_linux_amd64 in archive"))?;
        let binary_data = extractor.extract(&exe_member)?;

        let (completions, man_pages) = with_temp_exe("dasel", &binary_data, |exe_path| {
            let completions = gen_completions_subcommand("dasel", &binary_data, "completion")?;
            let man_data = run_cmd(exe_path, &["man"])?;
            Ok((completions, vec![ManPage::new(1, "dasel.1", man_data)]))
        })?;

        Ok(DownloadedAssets {
            binary: Some(AppBinary::new("dasel", binary_data)),
            completions,
            man_pages,
            ..Default::default()
        })
    }
}
