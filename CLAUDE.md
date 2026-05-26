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
  main.rs           # entry point: env_logger init, dispatches to lib functions
  lib.rs            # public API
  apps/
    app_trait.rs    # Trait for all apps that can be downloaded and installed by `relget`
    app_factory.rs  # App implementations are kept private to `apps`. Public API exposes
                    # `str` identifiers through which `App` instances can be instantiated
    app_registry.rs # static data containing all implemented apps and thin wrappers
                    # for accessing it
    containers/    # d4s, dock_mate, dry, lazydocker
    data/          # dasel, fx, gojq, jid, jq, jqp, qsv, qsv_all, rsv, xq, yq
    dev_envs/      # aqua, fnm, go, mise, uv
    dev_tools/     # ast_grep, mdbook, neovide, rust_analyzer, stylua
    files/         # bat, dust, eza, fd_find, ripgrep, sd_edit, yazi
    git/           # delta, difftastic, gitleaks, lazygit, mergiraf
    http/          # caddy, restish, xh
    logs/          # gonzo, lazy_journal
    shell/         # atuin, carapace, fzf, skim, starship, zoxide
    other/         # rclone, chezmoi
  cli/             # clap CLI structs: Cli, Commands and functions that implement them
  clients/         # GitHub and Codeberg clients, caching
    github.rs      # GithubClient with singleton Lazy<Mutex<GhCache>>
    codeberg.rs    # CodebergClient with singleton Lazy<Mutex<GhCache>>
    cache.rs       # GhCache: memory HashMap + disk under ~/.cache/relget/
  archive.rs       # ArchiveExtractor: .tar.gz/.tar.bz2/.tar.xz/.tar/.zip/.deb/.gz
  installer.rs     # install_assets(), with_temp_exe(), run_cmd(), gen_completions_*()
  uninstaller.rs   # uninstall_app(): removes binary, completions, man pages
  types.rs         # AppBinary, ManPage, Shell, Completion, DownloadedAssets
  version.rs       # AppVersion(u64, u64, u64) with find_in(), parse(), Display, Ord
```

## Adding a new GitHub app

1. Create `src/apps/<category>/myapp.rs` implementing the `App` trait:
2. Register in `src/apps/<category>/mod.rs`: add `mod myapp;` and `pub use myapp::MyApp;`
3. Register in `src/apps/apps_registry.rs` creating new `AppEntry` in `ALL_APP_ENTRIES`
4. Update `create_app()` in `src/apps/apps_factory.rs`, add `"myapp" => Some(Box::new(myapp::MyApp::new(client)))`
5. run `cargo xtask update-readme` so that `README.md` gets updated with new app

## Adding a new Codeberg app

Same as above but use `CodebergClient` instead of `GithubClient`:

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

// Generic: `<exe> [prefix_args...] <shell>` — basis for the helpers above:
gen_completions_with_shell_arg("myapp", &data, &["subcommand"])

// Run any command against a temp-installed binary:
with_temp_exe("myapp", &data, |path| { ... })
```

## Token handling

Tokens are optional. Without them, `relget` works anonymously (subject to API rate limits).

Config file `~/.config/relget.toml` (optional):
```toml
github_token = "ghp_..."
codeberg_token = "..."
```

Env vars override the config file (higher precedence):
- `RELGET_GHB_TOKEN` — GitHub token
- `RELGET_CDB_TOKEN` — Codeberg token

CLI flags:
- `--gh-token-source prompt`: prompts on stdin (masked)
- `--gh-token-source load` (default): reads `RELGET_GHB_TOKEN` env, then `~/.config/relget.toml`
- `--cb-token-source prompt`: prompts on stdin (masked)
- `--cb-token-source load` (default): reads `RELGET_CDB_TOKEN` env, then `~/.config/relget.toml`

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
