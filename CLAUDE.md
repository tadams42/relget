# relget

Rust CLI that installs/updates CLI utilities directly from GitHub and Codeberg releases.

## Build & run

```sh
cargo build
cargo run -- --help
cargo run -- --apps rg --gh-token-source load
cargo run -- --apps rg,bat --gh-token-source load   # comma-separated
cargo run -- list-apps-ids
cargo run -- completions zsh
cargo run -- uninstall --apps rg
cargo run -- reinstall --apps rg
cargo run -- --offline                              # use only cached data
```

Use `--prefix /tmp/try-relget/` to avoid needing `sudo` during local testing.

## Architecture

```
src/
  main.rs          # entry point: env_logger init, dispatches to lib functions
  cli.rs           # clap CLI structs: Cli, Commands (ListAppsIds, Completions, Uninstall, Reinstall)
  lib.rs           # public API: install_apps(), uninstall_apps(), select_apps(),
                   #   resolve_github_token(), resolve_codeberg_token(),
                   #   known_apps_identifiers(), MINIMAL_SET
  apps/
    mod.rs         # App trait + ALL_APP_ENTRIES static slice + all_app_entries() + create_app()
    chezmoi.rs     # top-level app file (not categorized into a subdir)
    rclone.rs      # top-level app file
    containers/    # d4s, dock_mate, dry, lazydocker
    data/          # dasel, fx, gojq, jid, jq, jqp, qsv, qsv_all, rsv, xq, yq
    dev_envs/      # aqua, fnm, go, mise, uv
    dev_tools/     # ast_grep, mdbook, neovide, rust_analyzer, stylua
    files/         # bat, dust, eza, fd_find, ripgrep, sd_edit, yazi
    git/           # delta, difftastic, gitleaks, lazygit, mergiraf
    http/          # caddy, restish, xh
    logs/          # gonzo, lazy_journal
    shell/         # atuin, carapace, fzf, skim, starship, zoxide
  github.rs        # GithubClient with singleton Lazy<Mutex<GhCache>>
  codeberg.rs      # CodebergClient with singleton Lazy<Mutex<GhCache>>
  cache.rs         # GhCache: memory HashMap + disk under ~/.cache/relget/
  archive.rs       # ArchiveExtractor: .tar.gz/.tar.bz2/.tar.xz/.tar/.zip/.deb/.gz
  installer.rs     # install_assets(), with_temp_exe(), run_cmd(), gen_completions_*()
  uninstaller.rs   # uninstall_app(): removes binary, completions, man pages
  types.rs         # AppBinary, ManPage, Shell, Completion, DownloadedAssets
  version.rs       # AppVersion(u64, u64, u64) with find_in(), parse(), Display, Ord
```

## Adding a new GitHub app

1. Create `src/apps/<category>/myapp.rs` implementing the `App` trait:

```rust
use std::sync::Arc;
use crate::apps::App;
use crate::github::GithubClient;
use crate::types::{AppBinary, DownloadedAssets};
use crate::version::AppVersion;
use anyhow::{Result, anyhow};

pub struct MyApp { client: Arc<GithubClient> }

impl MyApp {
    pub const URL: &'static str = "https://github.com/owner/repo";
    pub const DESCRIPTION: &'static str = "Short description of the tool";
    const OWNER: &'static str = "owner";
    const REPO: &'static str = "repo";
    pub fn new(client: Arc<GithubClient>) -> Self { Self { client } }
}

impl App for MyApp {
    fn exe_name(&self) -> &str { "myapp" }

    fn released_version(&self) -> Result<AppVersion> {
        self.client.latest_release(Self::OWNER, Self::REPO)?.version()
    }

    fn download(&self) -> Result<DownloadedAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release.asset_names().into_iter()
            .find(|a| a.contains("x86_64") && a.contains("linux") && a.ends_with(".tar.gz"))
            .ok_or_else(|| anyhow!("Can't find asset"))?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        // extract binary from archive, optionally generate completions
        Ok(DownloadedAssets { binary: Some(AppBinary::new("myapp", data)), ..Default::default() })
    }
}
```

2. Register in `src/apps/<category>/mod.rs`: add `pub mod myapp;`

3. Register in `src/apps/mod.rs`:
   - Add `AppEntry { id: "myapp", url: myapp::MyApp::URL, category: "<category>", description: myapp::MyApp::DESCRIPTION }` to `ALL_APP_ENTRIES`
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
"myapp" => Some(Box::new(myapp::MyApp::new(Arc::new(CodebergClient::new(cb_token, offline))))),
```

## App trait defaults

- `cli_version_arg()` → `"--version"`
- `installed_version()` runs `<exe> <cli_version_arg>`, combines stdout+stderr, calls `AppVersion::find_in()` on the result; override if the app's version output needs special handling
- `install()` calls `needs_install()` → `download()` → `install_assets()`; only override `download()`

## Installer helpers

```rust
// Generate completions: `<exe> <subcommand> <shell>` (e.g. "starship completions zsh"):
gen_completions_subcommand("myapp", &data, "completions")

// Generate completions: `<exe> <subcommand> <flag> <shell>` (e.g. "atuin gen-completions --shell zsh"):
gen_completions_shell_flag("myapp", &data, "gen-completions", "--shell")

// Generate completions with arbitrary per-shell flags (e.g. "--zsh", "--bash", "--fish"):
gen_completions_with_flags("myapp", &data, "--zsh", "--bash", "--fish")

// Generic: `<exe> [prefix_args...] <shell>` — basis for the helpers above:
gen_completions_with_shell_arg("myapp", &data, &["subcommand"])

// Run any command against a temp-installed binary:
with_temp_exe("myapp", &data, |path| { ... })
```

## Token handling

- `--gh-token-source prompt`: prompts on stdin (masked)
- `--gh-token-source load` (default): reads `GITHUB_API_TOKEN` env, then `~/.config/github/api_token`
- `--cb-token-source prompt`: prompts on stdin (masked)
- `--cb-token-source load` (default): reads `CODEBERG_API_TOKEN` env, then `~/.config/codeberg/api_token`

`CodebergClient::new(token, offline)` uses the provided token if `Some`, otherwise falls back to auto-loading from env/file.

## Cache

`GhCache` is reused for both GitHub and Codeberg:
- GitHub: `~/.cache/relget/{owner}/{repo}/release.json` and `asset.{id}`
- Codeberg: `~/.cache/relget/codeberg/{owner}/{repo}/release.json` and `asset.{id}`
- Release cache TTL: 1 day
- Asset cache: permanent (keyed by asset ID, which changes when a new release is published)

## Special cases

- `dev_envs/go.rs`: does not use GithubClient; downloads a fixed version from go.dev, extracts a directory to `prefix/go/`, symlinks `prefix/bin/go`
- `dev_envs/uv.rs`: installs two binaries (`uv` + `uvx`) and generates completions for both
- Many apps include man pages — use `ManPage::new(section, filename, data)` in `DownloadedAssets`

## Code conventions

- use `cargo +nightly fmt` to format the code
- use `cargo check --workspace` and `cargo clippy --no-deps` to lint the code
