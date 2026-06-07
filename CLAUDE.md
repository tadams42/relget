# relget

Rust CLI that installs/updates CLI utilities directly from GitHub and Codeberg releases.

## Build & run

```sh
cargo build
cargo run -- --help
cargo run -- install --apps rg
cargo run -- install --apps rg,bat        # comma-separated
cargo run -- list-apps-ids
cargo run -- completions zsh
cargo run -- uninstall --apps rg
cargo run -- install --apps rg --offline  # use only cached data
```

Use `--prefix tmp/try-relget/` to avoid needing `sudo` during local testing.

## Architecture

```
src/
  main.rs           # entry point: env_logger init, dispatches to lib functions
  lib.rs            # public API
  apps/
    app_trait.rs    # App trait: exe_name, released_version, assets, download, install
    app_factory.rs  # App implementations are kept private to `apps`. Public API exposes
                    # `str` identifiers through which `App` instances can be instantiated
    apps_registry.rs # loads registry.yaml at startup (OnceLock); exposes all_app_entries()
                    # and MINIMAL_SET
    registry.yaml   # source of truth for app metadata: id, url, category, description;
                    # embedded at compile time via rust-embed, parsed with serde_yaml
    containers/    # d4s, dock_mate, dry, lazydocker
    data_processing/ # dasel, fx, gojq, jid, jq, jqp, qsv, qsv_all, rsv, xq, yq
    databases/     # pdot, pgplan, sabiql, squix, usql
    docs_diag/     # mdbook, tlrc
    dev_envs/      # aqua, chezmoi, fnm, mise, uv
    coding/        # ast_grep, neovide, rust_analyzer, scc, stylua
    encryption/    # age
    files/         # bat, eza, f2, fd_find, rclone, ripgrep, sd_edit, trash, yazi
    git/           # delta, difftastic, gitleaks, lazygit, mergiraf
    http/          # caddy, hurl, restish, xh
    logs/          # gonzo, lazy_journal, logdy
    music/         # spotatui
    shell/         # atuin, carapace, fzf, skim, starship, zoxide
    system/        # dust, dysk, procs
  cli/             # clap CLI structs: Cli, Commands and functions that implement them
  clients/         # GitHub, Codeberg, and GitLab clients, caching
    github.rs      # GithubClient with singleton Lazy<Mutex<RelgetCache>>
    codeberg.rs    # CodebergClient with singleton Lazy<Mutex<RelgetCache>>
    gitlab.rs      # GitlabClient with singleton Lazy<Mutex<RelgetCache>>
    cache.rs       # memory HashMap + disk under ~/.cache/relget/
  archive.rs       # ArchiveExtractor: .tar.gz/.tar.bz2/.tar.xz/.tar/.zip/.deb/.gz
  installer.rs     # install_assets(), with_temp_exe(), run_cmd(), gen_completions_*()
  uninstaller.rs   # uninstall_app(): calls app.assets() to remove exactly those paths
  types.rs         # AppBinary, ManPage, Shell, Completion, AppAssets
  version.rs       # AppVersion(u64, u64, u64) with find_in(), parse(), Display, Ord
```

## Adding a new app

GitHub app:

1. Create `src/apps/<category>/myapp.rs` implementing the `App` trait:
   - Declare `pub const ID: &'static str = "myapp"` and `const EXE_NAME: &'static str = "myapp"`
     in the impl block. Do NOT add URL, CATEGORY, or DESCRIPTION constants — those live in
     `registry.yaml` only.
   - Implement `fn assets(&self) -> AppAssets` returning a static descriptor of every file the
     app installs. Use `Self::EXE_NAME` for the primary binary name:
     ```rust
     fn assets(&self) -> AppAssets {
         AppAssets {
             binary:      Some(AppBinary::descriptor(Self::EXE_NAME)),
             man_pages:   vec![ManPage::descriptor(1, "myapp.1")],
             completions: vec![
                 Completion::zsh_desc(Self::EXE_NAME),
                 Completion::bash_desc(Self::EXE_NAME),
                 Completion::fish_desc(Self::EXE_NAME),
             ],
             ..Default::default()
         }
     }
     ```
     Secondary binaries with different names (e.g. `"hurlfmt"`, `"uvx"`) must use hardcoded
     strings. `assets()` is the source of truth for the uninstaller — it must exactly match
     every file that `download()` installs.
   - Implement `fn download(&self) -> Result<AppAssets>` whose returned asset set must match
     `assets()` exactly (same files, same names).
2. Register in `src/apps/<category>/mod.rs`: add `mod myapp;` and `pub use myapp::MyApp;`
3. Add an entry to `src/apps/registry.yaml`:
   ```yaml
   - id: myapp
     exe_name: myapp
     url: https://github.com/owner/repo
     category: <category>
     description: One-line description of what the app does
     has_musl: false
     man_pages: unavailable        # unavailable | bundled | self_generated
     shell_completions: unavailable # unavailable | bundled | self_generated
   ```
4. Update `create_app()` in `src/apps/apps_factory.rs`: add
   `"myapp" => Some(Box::new(myapp::MyApp::new(client)))`
5. Commit the changes first, then run `cargo xtask update-docs`, then amend the commit.
   `xtask update-docs` reads git log to build `CHANGELOG.md`, so the new commit must
   exist before xtask runs — otherwise the new app won't appear in the changelog. The
   amend folds the updated `CHANGELOG.md` (and `SUPPORTED_APPS.md`) back into the same
   commit. We prefer generating single commit per each app added.

Codeberg app:

Same as above but use `CodebergClient` instead of `GithubClient`.

GitLab app:

Same as above but use `GitlabClient` instead of `GithubClient`. Note that GitLab release assets are stored under `assets.links[].{id, name, direct_asset_url}` in the API response; `GitlabClient` normalizes this to the same shape as GitHub/Codeberg before storing in `GhRelease`, so the same `release.asset_names()` / `release.asset_download_url()` helpers work.

When choosing build artifact to download for new app, use the one for Linux, `x86_64` architecture.

Prefer `musl` builds if available.

If downloaded artifact contains `man` pages, and app can also generate them, prefer generating them by running downloaded app. Same goes for shell completions: prefer running the app and generating them. Of course, if they can't be generated, use the ones shipped in build artifact.

When installing shell completions, get the ones for `Bash`, `Fish` and `ZSH`. Others don't interest us.

Sometimes app provides both `.deb` and `.tar.gz` build artifact, but `.tar.gz` doesn't contain man pages or shell completions, and they can't be generated on the fly. In this case, download and extract `.deb` too, it sometimes includes missing pieces from `.tar.gz`

Note that you probably need to download and extract app binary to be able to check if it can self-generate man pages or shell completions.

Always verify which argument the binary uses to report its version. Run it with `--version`, `version`, and `-v` to find out which one works. The `App` trait defaults to `--version`; if the app uses anything else, override `cli_version_arg()`:

```rust
fn cli_version_arg(&self) -> &str { "version" }  // subcommand style
fn cli_version_arg(&self) -> &str { "-v" }        // short flag style
```

Getting this wrong causes `installed_version()` to return `None`, which makes `needs_install()` always true and the app reinstalled on every `update` run.

## Removing an app

1. remove it's implementation
2. Commit the changes first, then run `cargo xtask update-docs`, then amend the commit.
   `xtask update-docs` reads git log to build `CHANGELOG.md`, so the new commit must
   exist before xtask runs — otherwise the new app won't appear in the changelog. The
   amend folds the updated `CHANGELOG.md` (and `SUPPORTED_APPS.md`) back into the same
   commit. We prefer generating single commit per each app removed.

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

`relget` by default tries to load tokens from config file. You may assume that this config file exists and is correctly set up in local development environment. You don't need to provide tokens in any way when calling `relget` for local testing.

## Cache

`RelgetCache` is reused for GitHub, Codeberg, and GitLab:
- GitHub: `~/.cache/relget/{owner}/{repo}/release.json` and `asset.{id}`
- Codeberg: `~/.cache/relget/codeberg/{owner}/{repo}/release.json` and `asset.{id}`
- GitLab: `~/.cache/relget/gitlab/{owner}/{repo}/release.json` and `asset.{id}`
- Release cache TTL: 1 day
- Asset cache: permanent (keyed by asset ID, which changes when a new release is published)

## Special cases

- `dev_envs/uv.rs`: installs two binaries (`uv` + `uvx`) and generates completions for both
- `http/hurl.rs`: installs `hurl` + `hurlfmt`, each with their own man pages and completions
- `coding/ast_grep.rs`: installs `ast-grep` + `sg`, each with their own completions
- `files/yazi.rs`: installs `yazi` + `ya`, each with their own completions
- `encryption/age.rs`: installs `age` + `age-keygen` (no completions)
- `data/qsv_all.rs`: installs `qsv` plus many variant binaries (`qsvdp`, `qsvlite`, etc.)

## Code conventions

- use `cargo +nightly fmt` to format the code
- use `cargo check --workspace` and `cargo clippy --no-deps` to lint the code
- git commit messages should use past tense (`added foobar` instead of `add foobar`,
  `adding foobar` or `adds foobar`)
- git commit messages should be prefixed by short category like `refactor:`, `build:`,
  `ci:`, `feat:`, `docs:`, `chore:` and similar
- git commit messages prefixed by `build:`, `ci:`, `docs:`, `refactor:` and `chore:`
  will be filtered from `CHANGELOG.md` so use these prefixes for stuff that is not
  interesting to end users
