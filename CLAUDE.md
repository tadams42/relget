# relget

Rust CLI that installs/updates CLI utilities directly from GitHub, GitLab and Codeberg releases.

## Build & run

```sh
cargo build
cargo run -- --help
cargo run -- install --apps rg
cargo run -- install --apps rg,bat        # comma-separated
cargo run -- registry list-apps-ids
cargo run -- completions zsh
cargo run -- uninstall --apps rg
cargo run -- install --apps rg --offline  # use only cached data
```

Use `--prefix tmp/try-relget/` to avoid needing `sudo` during local testing.

## Adding a new app

To add support for a new app, see the **[Contributing a new app](README.md#contributing-a-new-app)**
section in `README.md`. All app definitions live in `src/registry/<letter>/<app-id>.jsonc` — no
Rust code is required.

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

## Code conventions

- use `cargo +nightly fmt` to format the code
- use `cargo check --workspace` and `cargo clippy --no-deps` to lint the code
- `cargo clippy` skips re-linting files unchanged since the last build (incremental cache); to get a
  fresh lint of all local source without recompiling dependencies use
  `cargo clean -p relget && cargo clippy --no-deps`
- git commit messages should use past tense (`added foobar` instead of `add foobar`,
  `adding foobar` or `adds foobar`)
- git commit messages should be prefixed by short category like `refact:`, `build:`,
  `ci:`, `feat:`, `docs:`, `chore:` and similar

We use `cargo xtask update-docs` to keep `CHANGELOG.md` and `SUPPORTED_APPS.md` up to date. This means that after each `git commit` you should run `cargo xtask update-docs` and then fold changes into that latest commit. Note that our `xtask` doesn't put all commits into `CHANGELOG.md`: the ones with prefixes defined `NOISE_PREFIXES` in `xtask/main.rs` are skipped.
