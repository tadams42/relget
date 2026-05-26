# relget

Downloads and installs bunch of cmdline utilities directly from `GitHub` and `Codeberg`
releases into `/usr/local`. Installs app itself, `man` pages and completion for `ZSH`,
`Bash` and `Fish` into standard locations.

**TL;DR**

```sh
curl -fsSL https://github.com/tadams42/relget/releases/latest/download/relget-x86_64-linux.tar.gz | tar xz

sudo ./relget --gh-token-source load --minimal-set

rm ./relget
```

## Why?

Whenever I need to `ssh` to some new VM, I usually loose access to my favorite
collection of CLI tools. Sometimes `sudo apt install ...` or similar can help. Often
times it can't.

`relget` always works ... though downloaded binaries may not 😎. The risk is acceptable
to me `99.999%` of times. `YMMV`.

Installing into `/usr/local` doesn't interfere with the rest of the system. Ie. you can
have `ripgrep` installed from both, official distro package and from `relget`: updating
any of them will not overwrite the other. Which one gets used when you call `ripgrep`
from your shell, depends on your `$PATH`. In most modern distros, stuff from
`/usr/local` has priority.

## Non goals and limitations

- `relget` is **NOT** a fully blown package manager
- there is no way to select the version of installed binary - `relget` **ALWAYS**
  installs latest version of each supported app:
  - ⚠️ which may not work on your current system (usually if your system is too old)
  - ⚠️ or that app's released binary is broken
  - ⚠️ or the binary works but the latest version that `relget` had just installed, is no
    longer compatible with whatever you have on your system that depends on it; and
    `relget` had just this moment installed it to location that makes sure this broken
    version is called by everything 😎
  - or ...
- `relget uninstall` is best-effort: it removes the primary binary, standard
  completions, and man pages matched by name, but does not guarantee complete
  removal (see `relget uninstall --help` for details)
- `relget` works on and installs utilities for **Linux only**; you may be able to make it
  work on some other systems, but it was never intended to be used like that [^1]
- `relget` downloads only `x86_64` binaries

## How to use it?

```sh
# install everything into /usr/local
sudo relget

# ... or install everything into ~/.local
relget --prefix ~/.local

# install a subset of apps
relget --prefix ~/.local --apps rg,bat,fzf

# install the hand-picked minimal set
relget --prefix ~/.local --minimal-set
```

`GitHub` applies rate limiting to unauthenticated API requests. Providing your own token
avoids hitting those limits. Create `~/.config/relget.toml`:

```toml
github_token = "ghp_..."
codeberg_token = "..."   # optional, only needed for Codeberg apps
```

Or export environment variables (these take precedence over the config file):

```sh
export RELGET_GHB_TOKEN="ghp_..."
export RELGET_CDB_TOKEN="..."
```

You can also supply a token interactively:

```sh
# prompt for GitHub token
relget --gh-token-source prompt

# prompt for Codeberg token
relget --cb-token-source prompt
```

## Uninstall

```sh
# uninstall specific apps
relget uninstall --prefix ~/.local --apps rg,bat

# uninstall the minimal set
relget uninstall --prefix ~/.local --minimal-set

# force a clean reinstall (uninstall + install in one step)
relget reinstall --prefix ~/.local --apps rg --gh-token-source load
```

`uninstall` is best-effort: it removes the primary binary, standard shell
completions, and man pages whose names match `{exe}-{anything}.N[.gz]`. Apps
that install additional binaries under different names (e.g. `uv` also installs
`uvx`) will have only the primary binary removed. Run `relget uninstall --help`
for the full list of caveats.

## Caching

`relget` always uses `~/.cache/relget` for stuff downloaded from `GitHub` and `Codeberg`.

## Supported apps

### Containers

- [d4s](https://github.com/jr-k/d4s) — Docker socket proxy for restricting container API access
- [dockmate](https://github.com/shubh-io/DockMate) — Terminal UI for managing Docker containers and images
- [dry](https://github.com/moncho/dry) — Interactive terminal application for Docker management
- [lazydocker](https://github.com/jesseduffield/lazydocker) — Terminal UI for Docker containers, images, and compose

### Data

- [dasel](https://github.com/TomWright/dasel) — Query and modify data in JSON, YAML, TOML, XML, and CSV
- [fx](https://github.com/antonmedv/fx) — Terminal JSON viewer and interactive processor
- [gojq](https://github.com/itchyny/gojq) — Pure Go implementation of jq with extended features
- [jid](https://github.com/simeji/jid) — Interactive JSON incremental digger
- [jq](https://github.com/jqlang/jq) — Lightweight command-line JSON processor
- [jqp](https://github.com/noahgorstein/jqp) — TUI playground for crafting jq queries
- [qsv](https://github.com/dathere/qsv) — High-performance CSV data-wrangling toolkit
- [qsv-all](https://github.com/dathere/qsv) — High-performance CSV data-wrangling toolkit (all variants)
- [rsv](https://github.com/ribbondz/rsv) — High-performance CSV/TSV toolkit for data exploration
- [xq](https://github.com/sibprogrammer/xq) — Command-line XML and HTML processor using XPath
- [yq](https://github.com/mikefarah/yq) — Portable command-line YAML, JSON, XML, and CSV processor

### Dev envs

- [aqua](https://github.com/aquaproj/aqua) — Declarative CLI tool installer and version manager
- [fnm](https://github.com/Schniz/fnm) — Fast and simple Node.js version manager
- [mise](https://github.com/jdx/mise) — Polyglot tool version manager and task runner
- [uv](https://github.com/astral-sh/uv) — Extremely fast Python package and project manager

### Dev tools

- [ast-grep](https://github.com/ast-grep/ast-grep) — Fast code search, lint, and rewriting using AST patterns
- [mdbook](https://github.com/rust-lang/mdBook) — Create books from Markdown source files
- [neovide](https://github.com/neovide/neovide) — GPU-accelerated GUI frontend for Neovim
- [rust-analyzer](https://github.com/rust-lang/rust-analyzer) — Rust language server providing IDE integration
- [scc](https://github.com/boyter/scc) — Fast and accurate code counter with complexity calculations
- [stylua](https://github.com/JohnnyMorganz/stylua) — Opinionated Lua code formatter

### Files

- [bat](https://github.com/sharkdp/bat) — cat clone with syntax highlighting and git integration
- [dust](https://github.com/bootandy/dust) — Intuitive disk usage visualizer, a du alternative
- [eza](https://github.com/eza-community/eza) — Modern replacement for ls with icons and git integration
- [f2](https://github.com/ayoisaiah/f2) — Cross-platform batch file renaming tool
- [fd](https://github.com/sharkdp/fd) — Simple, fast, user-friendly alternative to find
- [ripgrep](https://github.com/BurntSushi/ripgrep) — Recursive regex search, a faster grep (ripgrep)
- [sd](https://github.com/chmln/sd) — Intuitive find-and-replace command, a sed alternative
- [trash-cli-rs](https://github.com/orf/trash) — Safe rm replacement that moves files to the trash
- [yazi](https://github.com/sxyazi/yazi) — Blazing fast terminal file manager with asynchronous I/O

### Git

- [delta](https://github.com/dandavison/delta) — Syntax-highlighting pager for git, diff, and grep output
- [difftastic](https://github.com/Wilfred/difftastic) — Structural diff tool that understands code syntax
- [gitleaks](https://github.com/gitleaks/gitleaks) — Detect secrets and sensitive data in git repositories
- [lazygit](https://github.com/jesseduffield/lazygit) — Simple terminal UI for git commands
- [mergiraf](https://codeberg.org/mergiraf/mergiraf) — Syntax-aware merge driver for git

### HTTP

- [caddy](https://github.com/caddyserver/caddy) — Fast, automatic HTTPS web server with TLS
- [restish](https://github.com/rest-sh/restish) — CLI for interacting with REST-ish HTTP APIs
- [xh](https://github.com/ducaale/xh) — Friendly and fast HTTP client, HTTPie alternative

### Logs

- [gonzo](https://github.com/control-theory/gonzo) — Log viewer TUI for structured and plain-text logs
- [lazyjournal](https://github.com/Lifailon/lazyjournal) — TUI for browsing systemd journal and Docker logs

### music

- [spotatui](https://github.com/LargeModGames/spotatui) — Terminal UI for Spotify

### Other

- [chezmoi](https://github.com/twpayne/chezmoi) — Dotfiles manager across multiple machines
- [rclone](https://github.com/rclone/rclone) — rsync for cloud storage (S3, GDrive, Dropbox, and more)

### Shell

- [atuin](https://github.com/atuinsh/atuin) — Shell history search backed by SQLite with sync
- [carapace](https://github.com/carapace-sh/carapace-bin) — Multi-shell completion generator for command-line tools
- [fzf](https://github.com/junegunn/fzf) — General-purpose command-line fuzzy finder
- [skim](https://github.com/skim-rs/skim) — Fuzzy finder in Rust (skim)
- [starship](https://github.com/starship/starship) — Minimal, blazing-fast, infinitely customizable shell prompt
- [zoxide](https://github.com/ajeetdsouza/zoxide) — Smarter cd command with frecency-based directory jumping

[^1]: Previously, `relget` had been written in Python.
      Workflow that required me to deploy Python to be able to deploy `relget` to be
      able to deploy various CLI utilities was not one of my brightest ideas. Luckily,
      Claude was able to rewrite the whole thing in Rust so I was able to abandon that
      silly Python project. 😎
