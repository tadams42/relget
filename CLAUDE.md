# relget

Rust CLI that installs/updates CLI utilities directly from GitHub and Codeberg releases.

## Build & run

```sh
cargo build
cargo run -- --help
cargo run -- --apps rg --gh-token-source load
cargo run -- --apps rg,bat --gh-token-source load   # comma-separated
cargo run -- list-apps-ids
```

Use `--prefix /tmp/try-relget/` to avoid needing `sudo` during local testing.

## Architecture

```
src/
  main.rs          # clap CLI: --prefix, --apps, --gh-token-source, --cb-token-source, --minimal-set
  lib.rs           # public API: install_apps(), select_apps(), load_github_token(), load_codeberg_token()
  apps/
    mod.rs         # App trait + all_app_entries() registry + create_app() factory
    *.rs           # one file per app
  github.rs        # GithubClient with singleton Lazy<Mutex<GhCache>>
  codeberg.rs      # CodebergClient with singleton Lazy<Mutex<GhCache>>
  cache.rs         # GhCache: memory HashMap + disk under ~/.cache/relget/
  archive.rs       # ArchiveExtractor: .tar.gz/.tar.bz2/.tar.xz/.tar/.zip/.deb/.gz
  installer.rs     # install_assets(), with_temp_exe(), run_cmd(), gen_completions_*()
  types.rs         # AppBinary, ManPage, Shell, Completion, DownloadedAssets
  version.rs       # AppVersion(u64, u64, u64) with parse(), Display, Ord
```

## Adding a new GitHub app

1. Create `src/apps/myapp.rs` implementing the `App` trait:

```rust
use std::sync::Arc;
use crate::apps::App;
use crate::github::GithubClient;
use crate::types::{AppBinary, DownloadedAssets};
use crate::version::AppVersion;
use anyhow::Result;

pub struct MyApp { client: Arc<GithubClient> }

impl MyApp {
    const OWNER: &'static str = "owner";
    const REPO: &'static str = "repo";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for MyApp {
    fn exe_name(&self) -> &str { "myapp" }
    fn url(&self) -> &str { "https://github.com/owner/repo" }

    fn released_version(&self) -> Result<AppVersion> {
        self.client.latest_release(Self::OWNER, Self::REPO)?.version()
    }

    fn download(&self) -> Result<DownloadedAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release.asset_names().into_iter()
            .find(|a| a.contains("x86_64") && a.contains("linux") && a.ends_with(".tar.gz"))
            .ok_or_else(|| anyhow::anyhow!("Can't find asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        // extract binary from archive, optionally generate completions
        Ok(DownloadedAssets { binary: Some(AppBinary::new("myapp", data)), ..Default::default() })
    }
}
```

2. Register in `src/apps/mod.rs`:
   - Add `pub mod myapp;`
   - Add `AppEntry { id: "myapp", url: "https://github.com/owner/repo" }` to `all_app_entries()`
   - Add `"myapp" => Some(Box::new(myapp::MyApp::new(client)))` to `create_app()`

## Adding a new Codeberg app

Same as above but use `CodebergClient` instead of `GithubClient`:

```rust
use crate::codeberg::CodebergClient;
pub struct MyApp { client: Arc<CodebergClient> }
impl MyApp { pub fn new(client: Arc<CodebergClient>) -> Self { Self { client } } }
```

In `create_app()`:
```rust
"myapp" => Some(Box::new(myapp::MyApp::new(Arc::new(CodebergClient::new(cb_token))))),
```

## App trait defaults

- `installed_version_flag()` → `"--version"`
- `installed_version_word_index()` → `-1` (last word of output)
- `parse_installed_version()` uses the index above; override if the app's `--version` output is unusual
- `install()` calls `needs_install()` → `download()` → `install_assets()`; only override `download()`

## Installer helpers

```rust
// Generate completions by running `<exe> <subcommand> <shell>`:
gen_completions_subcommand("myapp", &data, "completion")

// Generate with a flag: `<exe> <subcommand> --<shell>`:
gen_completions_shell_flag("myapp", &data, "completions", "zsh")

// Arbitrary per-shell flags:
gen_completions_with_flags("myapp", &data, "--zsh", "--bash", "--fish")

// Run any command against a temp-installed binary:
with_temp_exe("myapp", &data, |path| { ... })
```

## Token handling

- `--gh-token-source prompt`: prompts on stdin
- `--gh-token-source load` (default): reads `GITHUB_API_TOKEN` env, then `~/.config/github/api_token`
- `--cb-token-source prompt`: prompts on stdin
- `--cb-token-source load` (default): reads `CODEBERG_API_TOKEN` env, then `~/.config/codeberg/api_token`

`CodebergClient::new(token)` uses the provided token if `Some`, otherwise falls back to auto-loading from env/file.

## Cache

`GhCache` is reused for both GitHub and Codeberg:
- GitHub: `~/.cache/relget/{owner}/{repo}/release.json` and `asset.{id}`
- Codeberg: `~/.cache/relget/codeberg/{owner}/{repo}/release.json` and `asset.{id}`
- Release cache TTL: 1 day
- Asset cache: permanent (keyed by asset ID, which changes when a new release is published)

## Special cases

- `go.rs`: does not use GithubClient; downloads from go.dev, extracts a directory to `prefix/go/`, symlinks `prefix/bin/go`
- `uv.rs`: installs two binaries (`uv` + `uvx`) and generates completions for both
- Apps with man pages: `caddy`, `dasel`, `eza`, `rclone` — use `ManPage::new(section, filename, data)`

## Code conventions

- use `cargo +nightly fmt` to format the code
- use `cargo check --workspace` and `cargo clippy --no-deps` to lint the code
