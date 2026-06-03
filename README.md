# relget

Downloads and installs bunch of cmdline utilities directly from `GitHub`, `Codeberg` and
`GitLab` releases and installs them into `/usr/local`. Installs app binaries, `man`
pages and completion for `ZSH`, `Bash` and `Fish` shells.

## TL;DR

```sh
curl -fsSL https://github.com/tadams42/relget/releases/latest/download/relget-x86_64-linux.tar.gz | tar xz

# install into ~/.local/bin ...
./relget --prefix ~/.local --apps bat,eza,fd,fnm,uv,jq,lazygit

# ... or install system wide into /usr/local
sudo ./relget --apps bat,eza,fd,fnm,uv,jq,lazygit

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
relget --prefix ~/.local --apps bat,eza,fd,fnm,uv,jq,lazygit

# install system wide into /usr/local/
sudo relget --apps bat,eza,fd,fnm,uv,jq,lazygit

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
relget --prefix ~/.local --configured-set work

# uninstall it
relget uninstall --prefix ~/.local --configured-set work
```

`--configured-set`, `--apps`, and `--minimal-set` are mutually exclusive — only one may
be given per invocation.

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

  Values from env variables have higher precedence than the ones read from config file.

- from `relget`'s interactive prompt

  ```sh
  # prompt for GitHub token
  relget --gh-token-source prompt

  # prompt for Codeberg token
  relget --cb-token-source prompt

  # prompt for GitLab token
  relget --gl-token-source prompt
  ```

## Supported apps

### Containers

- [d4s](https://github.com/jr-k/d4s) — Terminal UI for managing Docker containers and images
- [dockmate](https://github.com/shubh-io/DockMate) — Terminal UI for managing Docker containers and images
- [dry](https://github.com/moncho/dry) — Terminal UI for managing Docker containers and images
- [lazydocker](https://github.com/jesseduffield/lazydocker) — Terminal UI for Docker containers, images, and compose

### Data

- [dasel](https://github.com/TomWright/dasel) — Query and modify data in JSON, YAML, TOML, XML, and CSV
- [fx](https://github.com/antonmedv/fx) — Terminal JSON viewer and interactive processor
- [gojq](https://github.com/itchyny/gojq) — Pure Go implementation of `jq` with extended features
- [jid](https://github.com/simeji/jid) — Interactive JSON incremental digger
- [jq](https://github.com/jqlang/jq) — Lightweight command-line JSON processor
- [jqp](https://github.com/noahgorstein/jqp) — TUI playground for crafting `jq` queries
- [qsv](https://github.com/dathere/qsv) — High-performance CSV data-wrangling toolkit
- [qsv-all](https://github.com/dathere/qsv) — High-performance CSV data-wrangling toolkit (additional binaries optimized for specific workloads)
- [rsv](https://github.com/ribbondz/rsv) — High-performance CSV/TSV toolkit for data exploration
- [xq](https://github.com/sibprogrammer/xq) — Command-line XML and HTML processor using XPath
- [yq](https://github.com/mikefarah/yq) — Portable command-line YAML, JSON, XML, and CSV processor

### databases

- [pdot](https://gitlab.com/dmfay/pdot) — PostgreSQL schema visualizer using Graphviz/Mermaid
- [pgplan](https://github.com/JacobArthurs/pgplan) — CLI tool for visualizing and analyzing PostgreSQL query plans
- [sabiql](https://github.com/riii111/sabiql) — TUI client for PostgreSQL databases
- [squix](https://github.com/eduardofuncao/squix) — Interactive TUI for exploring and querying SQL databases
- [usql](https://github.com/xo/usql) — Universal CLI for PostgreSQL, MySQL, SQLite, and many other databases

### Dev envs

- [aqua](https://github.com/aquaproj/aqua) — Declarative CLI tool installer and version manager
- [fnm](https://github.com/Schniz/fnm) — Fast and simple Node.js version manager
- [mise](https://github.com/jdx/mise) — prepares your complete development environment before each command runs
- [uv](https://github.com/astral-sh/uv) — Extremely fast Python package and project manager

### Dev tools

- [ast-grep](https://github.com/ast-grep/ast-grep) — Fast code search, lint, and rewriting using AST patterns
- [mdbook](https://github.com/rust-lang/mdBook) — Create books from Markdown source files
- [neovide](https://github.com/neovide/neovide) — GPU-accelerated GUI frontend for Neovim
- [rust-analyzer](https://github.com/rust-lang/rust-analyzer) — Rust language server providing IDE integration
- [scc](https://github.com/boyter/scc) — Fast and accurate code counter with complexity calculations - better `cloc`
- [stylua](https://github.com/JohnnyMorganz/stylua) — Opinionated Lua code formatter

### encryption

- [age](https://github.com/FiloSottile/age) — Simple, modern and secure file encryption tool

### Files

- [bat](https://github.com/sharkdp/bat) — `cat` clone with syntax highlighting and git integration
- [dust](https://github.com/bootandy/dust) — Intuitive disk usage visualizer, a `du` alternative
- [dysk](https://github.com/Canop/dysk) — Terminal utility to get information on filesystems (`df` alternative)
- [eza](https://github.com/eza-community/eza) — Modern replacement for `ls` with icons and git integration
- [f2](https://github.com/ayoisaiah/f2) — Batch file renaming tool
- [fd](https://github.com/sharkdp/fd) — Simple, fast, user-friendly alternative to `find`
- [ripgrep](https://github.com/BurntSushi/ripgrep) — Recursive regex search, a faster `grep`
- [sd](https://github.com/chmln/sd) — Intuitive find-and-replace command, a `sed` alternative
- [trash-cli-rs](https://github.com/orf/trash) — Safe `rm` replacement that moves files to the trash
- [yazi](https://github.com/sxyazi/yazi) — Blazing fast terminal file manager with asynchronous I/O

### Git

- [delta](https://github.com/dandavison/delta) — Syntax-highlighting pager for git, diff, and grep output
- [difftastic](https://github.com/Wilfred/difftastic) — Structural diff tool that understands code syntax
- [gitleaks](https://github.com/gitleaks/gitleaks) — Detect secrets and sensitive data in git repositories
- [lazygit](https://github.com/jesseduffield/lazygit) — Simple terminal UI for `git` commands
- [mergiraf](https://codeberg.org/mergiraf/mergiraf) — Syntax-aware merge driver for `git`

### HTTP

- [caddy](https://github.com/caddyserver/caddy) — Fast, automatic HTTPS web server with TLS
- [hurl](https://github.com/Orange-OpenSource/hurl) — Run and test HTTP requests defined in plain text
- [restish](https://github.com/rest-sh/restish) — CLI for interacting with REST-ish HTTP APIs, `curl` alternative
- [xh](https://github.com/ducaale/xh) — Friendly and fast HTTP client, `HTTPie` and `curl` alternative

### Logs

- [gonzo](https://github.com/control-theory/gonzo) — Log viewer TUI for structured and plain-text logs
- [lazyjournal](https://github.com/Lifailon/lazyjournal) — TUI for browsing systemd journal and Docker logs
- [logdy](https://github.com/logdyhq/logdy-core) — Web-based real-time log viewer with filtering and search

### music

- [spotatui](https://github.com/LargeModGames/spotatui) — Terminal UI for Spotify

### Other

- [chezmoi](https://github.com/twpayne/chezmoi) — Dotfiles manager across multiple machines, think `ansible` but for dotfiles
- [rclone](https://github.com/rclone/rclone) — `rsync` for cloud storage (S3, GDrive, Dropbox, and more)
- [tlrc](https://github.com/tldr-pages/tlrc) — Official [tldr-pages](https://tldr.sh/) client written in Rust

### Shell

- [atuin](https://github.com/atuinsh/atuin) — Shell history search backed by SQLite with sync
- [carapace](https://github.com/carapace-sh/carapace-bin) — Multi-shell completion generator for command-line tools
- [fzf](https://github.com/junegunn/fzf) — General-purpose command-line fuzzy finder
- [skim](https://github.com/skim-rs/skim) — Fuzzy finder in Rust (alternative to `fzf`)
- [starship](https://github.com/starship/starship) — Minimal, blazing-fast, infinitely customizable shell prompt
- [zoxide](https://github.com/ajeetdsouza/zoxide) — Smarter `cd` command with frecency-based directory jumping

[^1]: Previously, `relget` had been written in Python.
      Workflow that required me to deploy Python to be able to deploy `relget` to be
      able to deploy various CLI utilities was not one of my brightest ideas. Luckily,
      Claude was able to rewrite the whole thing in Rust so I was able to abandon that
      silly Python project. 😎
