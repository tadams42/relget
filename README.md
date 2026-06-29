# relget

[![Latest release](https://img.shields.io/github/v/release/tadams42/relget)](https://github.com/tadams42/relget/releases/latest)
[![Release](https://github.com/tadams42/relget/actions/workflows/release.yml/badge.svg)](https://github.com/tadams42/relget/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
![Rust](https://img.shields.io/badge/rust-2021-orange)

Downloads and installs bunch of cmdline utilities directly from `GitHub`, `Codeberg` and
`GitLab` releases and installs them into `/usr/local`. Installs app binaries, `man`
pages and completion for `ZSH`, `Bash` and `Fish` shells.

## TL;DR

```sh
curl -fsSL https://github.com/tadams42/relget/releases/latest/download/relget-x86_64-unknown-linux-musl.tar.gz | tar xz

# install into ~/.local/bin ...
./relget install --prefix ~/.local --apps bat,eza,fd,fnm,uv,jq,lazygit

# ... or install system wide into /usr/local
sudo ./relget install --apps bat,eza,fd,fnm,uv,jq,lazygit

rm ./relget
```

## Why?

Whenever I need to `ssh` to some new VM, I usually lose access to my favorite collection
of CLI tools. Sometimes `sudo apt install ...` or similar can help. Often times it
can't: either the thing I want is not (yet) available in distro repos, or the available
version is ancient.

## Non goals and limitations

- `relget` is **NOT** a fully blown package manager
- ⚠️ there is no way to select the version of installed binary - `relget` **ALWAYS**
  installs latest version which:
  - ⚠️ may not work on your current (somewhat old) system
  - ⚠️ had not been tested by your distribution
- `relget` works on and installs utilities for **Linux only** [^1]
- `relget` installs only `x86_64` binaries

Installing into `/usr/local/` doesn't interfere with official distribution installer.
You can have `ripgrep` installed from both, official distro package and from `relget`.
Updating any of them will not overwrite the other.

But...

⚠️ Once you install into `/usr/local/`, this binary will usually be prioritized - not
the official one from distro. This can cause all sorts of unexpected problems.

The risks are acceptable to me `99.999%` of times. **YMMV**

## How to use it?

```sh
# install into ~/.local/bin
relget install --prefix ~/.local --apps bat,eza,fd,fnm,uv,jq,lazygit

# install system wide into /usr/local/
sudo relget install --apps bat,eza,fd,fnm,uv,jq,lazygit

# uninstall specific apps from specific prefix
relget uninstall --prefix ~/.local --apps rg,bat

# uninstall from /usr/local/
sudo relget uninstall --apps bat,eza,fd,fnm,uv,jq,lazygit

# update all apps already installed in prefix
relget update --prefix ~/.local

# update some of the apps in prefix
relget update --prefix ~/.local --apps rg,bat
```

### Named app sets

You can define reusable named lists of apps in `~/.config/relget.toml` under a `[sets]`
table:

```toml
github_token = "ghp_..."

[sets]
work = ["rg", "bat", "delta", "lazygit", "fzf", "fd-find"]
home = ["rg", "bat", "delta", "lazygit", "fzf", "fd-find", "starship", "zoxide"]
```

and select them with `--configured-set`:

```sh
# install the "work" set
relget install --prefix ~/.local --configured-set work

# uninstall it
relget uninstall --prefix ~/.local --configured-set work
```

`--configured-set` and `--apps` are mutually exclusive — only one may be given per
invocation.

### API Rate limits on GitHub (and others)

`relget` always uses `~/.cache/relget/` for stuff if downloads, so it is usually safe to
use it as-is. But, in some cases, you may hit rate limits on services `relget` uses. All
services raise these limits significantly if you access their APIs using `Personal
Access Tokens`. `relget` can be configured to read `PAT`s:

- from config file `~/.config/relget.toml`:

  ```toml
  github_token = "ghp_..."
  codeberg_token = "..."
  gitlab_token = "glpat-..."
  ```

- from environment variables:

  ```sh
  export RELGET_GHB_TOKEN="ghp_..."
  export RELGET_CDB_TOKEN="..."
  export RELGET_GLB_TOKEN="..."
  ```

  Env variables take precedence over the config file.

## Contributing a new app

Each supported app is described by a single JSONC file in the source tree. No Rust code is
required — the registry file is all you need.

### File location

```
src/registry/<first-letter-of-id>/<app-id>.jsonc
```

For example: `src/registry/b/bat.jsonc`. The file name (without `.jsonc`) must match the `id`
field and must be globally unique in the registry.

### Registry file structure

```jsonc
{
  // Required. Must match the filename (without .jsonc) and be globally unique.
  "id": "myapp",

  // Required. Must match a category id in `src/registry/categories.jsonc`.
  "category_id": "files",

  // Optional but encouraged.
  "description": "One-line description of what the app does",

  // Forge URL. Determines which API client is used automatically:
  // github.com → GithubClient, codeberg.org → CodebergClient, gitlab.com → GitlabClient
  "url": "https://github.com/owner/myapp",

  // One entry per binary that gets installed. At least one entry required.
  "binaries": [
    {
      "id": 1,              // Numeric id; referenced by shell_completions[].binary_id
                            // and man_pages[].binary_id entries.
      "name": "myapp",     // Exact binary filename as it appears in the release archive.
      "version_cmdline": "--version", // Arg(s) to pass to get the version string.
                            // Try --version, then version (subcommand), then -v.
      "is_main": true       // Exactly one binary must be true. Its version is used for
                            // update checks and its name is the installed exe name.
    }
    // Additional binaries (e.g. "uvx", "hurlfmt") follow the same shape with is_main: false.
  ],

  // One entry per release artifact to download. At least one entry required.
  "assets": [
    {
      "id": 1,              // Numeric id; referenced by shell_completions[].asset_id
                            // and man_pages[].asset_id entries.
      "type": "archive",   // "archive" (tar/zip), "deb", or "binary" (raw executable).
      // Matching conditions — all specified conditions must hold simultaneously.
      // Use the minimum set needed to uniquely identify the asset.
      "contains": "x86_64-unknown-linux-musl",
      "ends_with": ".tar.gz"
      // Other matchers: "starts_with", "not_contains", "equals".
      // Special sentinel: "equals": "tarball" downloads the repo source tarball.
    }
  ],

  // One entry per (shell, binary) completion pair. Empty array if none.
  "shell_completions": [
    // Option A: binary generates it on stdout.
    { "shell": "bash", "self_generated": { "binary_id": 1, "command": "completions bash" } },
    { "shell": "zsh",  "self_generated": { "binary_id": 1, "command": "completions zsh" } },
    { "shell": "fish", "self_generated": { "binary_id": 1, "command": "completions fish" } }
    // Option B: extract a file from a downloaded asset.
    // { "shell": "zsh", "extracted": { "asset_id": 1, "path": "_myapp" } }
  ],

  // One entry per man page. Empty array if none.
  "man_pages": [
    // Option A: binary generates it (stdout captured, named "<binary>.<section>").
    { "section": 1, "self_generated": { "binary_id": 1, "command": "man" } }
    // Option B: binary generates many pages into a directory — see {{ tmp-dir }} below.
    // Option C: extract a file from a downloaded asset.
    // { "section": 1, "extracted": { "asset_id": 1, "path": "myapp.1" } }
  ]

  // Optional. Omit if the default behavior works (latest release, version from tag).
  // "released_version_parse": { ... }  — see below
}
```

### `released_version_parse`

By default `relget` fetches the latest release and parses the version from its tag. Two fields
let you override that behavior:

```jsonc
"released_version_parse": {
  // Only consider releases whose tag starts with this prefix.
  // Use when the repo publishes nightly/pre-release tags (e.g. "nightly-20240101")
  // alongside stable ones (e.g. "v1.2.3") and you only want the stable ones.
  "tag_starts_with": "v",

  // Scan the release body for a version string before falling back to the tag.
  // Use when the tag format (e.g. CalVer "2026-05-18") is not the same value the
  // binary reports at runtime (e.g. "0.3.2904"). See rust-analyzer.jsonc for an example.
  "try_in_body": false
}
```

### `{{ tmp-dir }}` — batch man page generators

Some apps generate all their man pages at once by writing them to a directory:

```jsonc
{ "section": 8, "self_generated": { "binary_id": 1, "command": "manpage --directory {{ tmp-dir }}" } }
```

`relget` substitutes `{{ tmp-dir }}` (spelling is load-bearing — use a hyphen, not an underscore)
with a real temp directory path, runs the command, and collects every file written there.

When a batch generator is present, you must also list each expected man page as an `extracted`
entry. These entries are never actually downloaded — they exist only so the uninstaller knows
which files to remove. Any `asset_id` is fine for these metadata entries. See
`src/registry/c/caddy.jsonc` for a complete example.

### Preference rules

- **Architecture**: always choose the `x86_64` Linux artifact.
- **musl vs glibc**: prefer musl builds when available; point the asset matcher at the
  musl archive.
- **Generated vs extracted**: if the binary can self-generate man pages or shell completions,
  prefer `self_generated` — the output is always current. Only use `extracted` when the binary
  can't generate them.
- **Shell completions to include**: Bash, Fish, and Zsh only.
- **Using `.deb` for man pages / completions**: if the main archive doesn't include man pages
  or completions and the binary can't generate them, add a second `deb` asset and point the
  relevant `extracted` entries at it. The primary binary still comes from the tar.gz or musl
  archive. See `src/registry/d/dust.jsonc` for an example.
- **Version cmdline**: download the binary locally and try `--version`, then `version`
  (subcommand style), then `-v`. Use whichever works in `version_cmdline`.

### Validation

After adding the file:

```sh
cargo check --workspace
cargo run -- registry list-apps-ids   # your new id should appear
cargo run -- install --prefix tmp/try-relget/ --apps <id>
```

[^1]: Previously, `relget` had been written in Python.
      Workflow that required me to deploy Python to be able to deploy `relget` to be
      able to deploy various CLI utilities was not one of my brightest ideas. Luckily,
      Claude was able to rewrite the whole thing in Rust so I was able to abandon that
      silly Python project. 😎
