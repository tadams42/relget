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

[^1]: Previously, `relget` had been written in Python.
      Workflow that required me to deploy Python to be able to deploy `relget` to be
      able to deploy various CLI utilities was not one of my brightest ideas. Luckily,
      Claude was able to rewrite the whole thing in Rust so I was able to abandon that
      silly Python project. 😎
