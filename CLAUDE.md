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
cargo run -- --apps rg --offline                    # use only cached data
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
    dev_envs/      # aqua, fnm, mise, uv
    dev_tools/     # ast_grep, mdbook, neovide, rust_analyzer, stylua
    files/         # bat, dust, eza, fd_find, ripgrep, sd_edit, yazi
    git/           # delta, difftastic, gitleaks, lazygit, mergiraf
    http/          # caddy, restish, xh
    logs/          # gonzo, lazy_journal
    shell/         # atuin, carapace, fzf, skim, starship, zoxide
    other/         # rclone, chezmoi
  cli/             # clap CLI structs: Cli, Commands and functions that implement them
  clients/         # GitHub, Codeberg, and GitLab clients, caching
    github.rs      # GithubClient with singleton Lazy<Mutex<GhCache>>
    codeberg.rs    # CodebergClient with singleton Lazy<Mutex<GhCache>>
    gitlab.rs      # GitlabClient with singleton Lazy<Mutex<GhCache>>
    cache.rs       # GhCache: memory HashMap + disk under ~/.cache/relget/
  archive.rs       # ArchiveExtractor: .tar.gz/.tar.bz2/.tar.xz/.tar/.zip/.deb/.gz
  installer.rs     # install_assets(), with_temp_exe(), run_cmd(), gen_completions_*()
  uninstaller.rs   # uninstall_app(): deterministic removal via app.assets()
  types.rs         # AppBinary, ManPage, Shell, Completion, AppAssets
  version.rs       # AppVersion(u64, u64, u64) with find_in(), parse(), Display, Ord
```

## Adding a new app

GitHub app:

1. Create `src/apps/<category>/myapp.rs` implementing the `App` trait:
   - Implement `fn assets(&self) -> AppAssets` returning a static descriptor of all installed files
     (binary, other_bins, man_pages, completions) using the descriptor constructors:
     `AppBinary::descriptor("name")`, `ManPage::descriptor(1, "name.1")`,
     `Completion::zsh_desc("name")` / `bash_desc` / `fish_desc`
   - Implement `fn download(&self) -> Result<AppAssets>` whose returned asset set must match `assets()`
2. Register in `src/apps/<category>/mod.rs`: add `mod myapp;` and `pub use myapp::MyApp;`
3. Register in `src/apps/apps_registry.rs` creating new `AppEntry` in `ALL_APP_ENTRIES`
4. Update `create_app()` in `src/apps/apps_factory.rs`, add `"myapp" => Some(Box::new(myapp::MyApp::new(client)))`
5. run `cargo xtask update-readme` so that `README.md` gets updated with new app

Codeberg app:

Same as above but use `CodebergClient` instead of `GithubClient`:

GitLab app:

Same as above but use `GitlabClient` instead of `GithubClient`. Note that GitLab release assets are stored under `assets.links[].{id, name, direct_asset_url}` in the API response; `GitlabClient` normalizes this to the same shape as GitHub/Codeberg before storing in `GhRelease`, so the same `release.asset_names()` / `release.asset_download_url()` helpers work.

When choosing build artifact to download for new app, use the one for Linux, `x86_64` architecture.

Prefer `musl` builds if available.

If downloaded artifact contains `man` pages, and app can also generate them, prefer generating them by running downloaded app. Same goes for shell completions: prefer running the app and generating them. Of course, if they can't be generated, use the ones shipped in build artifact.

When installing shell completions, get the ones for `Bash`, `Fish` and `ZSH`. Others don't interest us.

Sometimes app provides both `.deb` and `.tar.gz` build artifact, but `.tar.gz` doesn't contain man pages or shell completions, and they can't be generated on the fly. In this case, download and extract `.deb` too, it sometimes includes missing pieces from `.tar.gz`

Note that you probably need to download and extract app binary to be able to check if it can self-generate man pages or shell completions.

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
gitlab_token = "..."
```

Env vars override the config file (higher precedence):
- `RELGET_GHB_TOKEN` — GitHub token
- `RELGET_CDB_TOKEN` — Codeberg token
- `RELGET_GLB_TOKEN` — GitLab token

CLI flags:
- `--gh-token-source prompt`: prompts on stdin (masked)
- `--gh-token-source load` (default): reads `RELGET_GHB_TOKEN` env, then `~/.config/relget.toml`
- `--cb-token-source prompt`: prompts on stdin (masked)
- `--cb-token-source load` (default): reads `RELGET_CDB_TOKEN` env, then `~/.config/relget.toml`
- `--gl-token-source prompt`: prompts on stdin (masked)
- `--gl-token-source load` (default): reads `RELGET_GLB_TOKEN` env, then `~/.config/relget.toml`

## Cache

`GhCache` is reused for GitHub, Codeberg, and GitLab:
- GitHub: `~/.cache/relget/{owner}/{repo}/release.json` and `asset.{id}`
- Codeberg: `~/.cache/relget/codeberg/{owner}/{repo}/release.json` and `asset.{id}`
- GitLab: `~/.cache/relget/gitlab/{owner}/{repo}/release.json` and `asset.{id}`
- Release cache TTL: 1 day
- Asset cache: permanent (keyed by asset ID, which changes when a new release is published)

## Special cases

- `dev_envs/uv.rs`: installs two binaries (`uv` + `uvx`) and generates completions for both

## Code conventions

- use `cargo +nightly fmt` to format the code
- use `cargo check --workspace` and `cargo clippy --no-deps` to lint the code
