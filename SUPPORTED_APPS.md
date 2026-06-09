# Supported apps

## Coding

Linters, formatters and various coding helpers.

- [ast-grep](https://github.com/ast-grep/ast-grep)
  Fast code search, lint, and rewriting using AST patterns

- [fend](https://github.com/printfn/fend)
  Arbitrary-precision unit-aware calculator

- [grex](https://github.com/pemistahl/grex)
  Generates regular expressions from user-provided test strings

- [hyperfine](https://github.com/sharkdp/hyperfine)
  Command-line benchmarking tool

- [mailpit](https://github.com/axllent/mailpit)
  Email and SMTP testing tool with a web interface to view captured emails

- [mkcert](https://github.com/FiloSottile/mkcert)
  Zero-config tool to make locally trusted development certificates

- [neovide](https://github.com/neovide/neovide)
  GPU-accelerated GUI frontend for Neovim

- [pyrefly](https://github.com/facebook/pyrefly)
  New, fast Python type checker from Meta

- [replibyte](https://github.com/Qovery/replibyte)
  Creates database dump from your production data and then restores it to your local development environment, replacing sensitive stuff with fake data.

- [rgx](https://github.com/brevity1swos/rgx)
  Regex debugger for the terminal — step-through execution, 3 engines, code generation. Similar to `https://regexr.com/` and `https://regex101.com/` but works locally, in TUI

- [ruff](https://github.com/astral-sh/ruff)
  Extremely fast Python linter and code formatter from authors of `uv`

- [rust-analyzer](https://github.com/rust-lang/rust-analyzer)
  Rust language server providing IDE integration

- [scc](https://github.com/boyter/scc)
  Fast and accurate code counter with complexity calculations - better `cloc`

- [sqruff](https://github.com/quarylabs/sqruff)
  Fast SQL formatter and linter

- [stylua](https://github.com/JohnnyMorganz/stylua)
  Opinionated Lua code formatter

- [taplo](https://github.com/tamasfe/taplo)
  Highly comprehensive, all-in-one TOML toolkit. It operates as a fully-featured Language Server Protocol (LSP), formatter, and linter. Natively supports JSON Schema validation applied to TOML files and extensive formatting configuration. `Even Better TOML` for VSCode is built on top of it.

- [tombi](https://github.com/tombi-toml/tombi)
  Modern, minimalist TOML linter and opinionated formatter. Not an LSP.

- [ty](https://github.com/astral-sh/ty)
  Extremely fast Python type checker from authors of `uv`. Currently in `beta`.

- [vacuum](https://github.com/daveshanley/vacuum)
  World's fastest OpenAPI 3 / Swagger linter and quality tool

## Containers

- [ctop](https://github.com/bcicen/ctop)
  Top-like interface for container metrics

- [d4s](https://github.com/jr-k/d4s)
  Terminal UI for managing Docker containers and images, browsing running
containers and viewing their logs. OP feature: a single key opens shell inside
of selected container.


- [dockmate](https://github.com/shubh-io/DockMate)
  Terminal UI for managing Docker containers and images

- [dry](https://github.com/moncho/dry)
  Terminal UI for managing Docker containers and images very similar to `d4s`

- [dtop](https://github.com/amir20/dtop)
  Terminal dashboard for Docker that monitors multiple hosts in real-time

- [lazydocker](https://github.com/jesseduffield/lazydocker)
  Terminal UI for Docker containers, images, and compose

## Data - processing JSON/YAML/CSV

CLI filters and processors for structured data. Some of them have TUI for interactive filtering and displaying structured data in terminal. Most of them have their own query DSL to quickly find relevant data in large files.

- [csvtk](https://github.com/shenwei356/csvtk)
  Cross-platform, efficient and practical CSV/TSV toolkit

- [dasel](https://github.com/TomWright/dasel)
  Query and modify data in JSON, YAML, TOML, XML, CSV, HCL, INI and KDL.

- [fq](https://github.com/wader/fq)
  `jq` for binary formats — tool, language and decoders for working with binary
and text formats


- [fx](https://github.com/antonmedv/fx)
  Interactive TUI for viewing JSON/YAML/TOML

- [gojq](https://github.com/itchyny/gojq)
  Pure Go implementation of `jq` with extended features

- [jaq](https://github.com/01mf02/jaq)
  Alternative to `jq` with fewer features but faster - for extremely large files.

- [jd](https://github.com/josephburnett/jd)
  diffing and patching JSON and YAML values. It supports a internal patch format,
`JSON Merge Patch` and a subset of `JSON Patch`.


- [jid](https://github.com/simeji/jid)
  JSON viewer with interactive entry and execution of `jq` filter expressions

- [jiq](https://github.com/bellicose100xp/jiq)
  Interactive JSON query tool with real-time output and AI assistance (similar to `jid` and `jqp`).

- [jq](https://github.com/jqlang/jq)
  Lightweight command-line JSON processor, golden standard for all other tools. Implements it's own query language for JSON

- [jqp](https://github.com/noahgorstein/jqp)
  JSON viewer with interactive entry and execution of `gojq` filter expressions (similar to `jid` but uses `gojq`)

- [jsongrep](https://github.com/micahkepe/jsongrep)
  Fast querying of `JSON`, `YAML`, `TOML`, `JSONL`, `CBOR` and `MessagePack` using path expressions.

- [miller](https://github.com/johnkerl/miller)
  Like `awk`, `sed`, `cut`, `join`, and `sort` for name-indexed data such as `CSV`, `TSV`, and tabular `JSON`.

- [qq](https://github.com/JFryy/qq)
  inspired by `jq`, multi-format processor (`JSON`, `YAML`, `TOML`, `XML`, `HCL`, `CSV`, `INI` and more)

- [qsv](https://github.com/dathere/qsv)
  High-performance CSV data-wrangling toolkit

- [qsv-all](https://github.com/dathere/qsv)
  High-performance CSV data-wrangling toolkit (additional binaries optimized for specific workloads). You probably don't need this - use `qsv` instead

- [rsv](https://github.com/ribbondz/rsv)
  CSV/TSV toolkit for data exploration, similar to `qsv` but smaller in scope and feature set.

- [tabiew](https://github.com/shshemi/tabiew)
  interactive viewing and filtering of tabular data (CSV, TSV, PSV, Parquet, and Arrow IPC)

- [tv](https://github.com/alexhallam/tv)
  (Tidy Viewer) pretty printer for tabular data (CSV, TSV, PSV, and Parquet).
Excels at numeric formatting and large data sets. Doesn't implement much of
filtering, but is quick and takes extra care on aligning (decimal) numbers for
better readability.


- [xan](https://github.com/medialab/xan)
  Command line tool for processing CSV files optimized to be extremely fast (forked from `BurntSushi/xsv` which is now archived)

- [xq](https://github.com/sibprogrammer/xq)
  Command-line XML and HTML processor using XPath

- [yq](https://github.com/mikefarah/yq)
  like `jq` but for YAML. In newer versions it can also work with JSON, XML and CSV

## Development environments

- [chezmoi](https://github.com/twpayne/chezmoi)
  Dotfiles manager across multiple machines, think `ansible` but for dotfiles

- [fnm](https://github.com/Schniz/fnm)
  Fast and simple Node.js version manager

- [uv](https://github.com/astral-sh/uv)
  Extremely fast Python package and project manager

## Documentation and diagrams

- [agg](https://github.com/asciinema/agg)
  Generate animated GIFs from `asciinema` session recordings

- [asciinema](https://github.com/asciinema/asciinema)
  Record and replay terminal sessions as lightweight `asciicast` files. These
files are not "video" but can be replayed as video. Additionally, they can also
be replayed as sequence of commands in terminal.


- [d2](https://github.com/terrastruct/d2)
  Powerful diagram scripting language that turns text to diagrams. More complex and capable than `mermaid`, less complex than `PlantUML`.

- [hugo](https://github.com/gohugoio/hugo)
  Fast and flexible static site generator

- [mdbook](https://github.com/rust-lang/mdBook)
  Create books from Markdown source files

- [pdot](https://gitlab.com/dmfay/pdot)
  PostgreSQL schema visualizer outputting Graphviz or Mermaid

- [pgplan](https://github.com/JacobArthurs/pgplan)
  CLI tool for visualizing and analyzing PostgreSQL query plans

- [tbls](https://github.com/k1LoW/tbls)
  CI-friendly tool to document a database schema

- [tlrc](https://github.com/tldr-pages/tlrc)
  Official [tldr-pages](https://tldr.sh/) client written in Rust

## Encryption and secrets management

- [age](https://github.com/FiloSottile/age)
  Simple, modern and secure file encryption tool

- [doppler](https://github.com/DopplerHQ/cli)
  Secrets manager CLI — sync env vars and secrets across teams, deployments and cloud providers.

- [gocryptfs](https://github.com/rfjakob/gocryptfs)
  Encrypted overlay filesystem written in Go. Fast, security-audited, actively
maintained. It encrypts files and obfuscates file and directory names.

It is usable for encrypting individual files and whole directories that are then
synced to various cloud storage providers (`Google Drive`, `OneDrive`, ...)
It doesn't hide overall count of files or structure of directories.

To achieve maximum privacy you can also use [CryFS](https://github.com/cryfs).
That one encrypts everything into series of equally sized blocks, completely
hiding folder structure. But, this is much(!!) slower and it doesn't play that
nicely with syncing to cloud storage.


- [rage](https://github.com/str4d/rage)
  Rust implementation of the `age` encryption tool

## Files

General tools for every day work with files in terminal.

- [bat](https://github.com/sharkdp/bat)
  `cat` clone with syntax highlighting and git integration


- [choose](https://github.com/theryangeary/choose)
  Human-friendly and fast alternative to `cut` (and sometimes `awk`)


- [eza](https://github.com/eza-community/eza)
  Modern replacement for `ls` with icons and git integration

- [f2](https://github.com/ayoisaiah/f2)
  Safe batch file renaming tool

- [fd](https://github.com/sharkdp/fd)
  Simple, fast, user-friendly alternative to `find`

- [rclone](https://github.com/rclone/rclone)
  `rsync` for cloud storage (S3, GDrive, Dropbox, and more)

- [ripgrep](https://github.com/BurntSushi/ripgrep)
  Recursive regex search, a faster `grep`

- [scooter](https://github.com/thomasschafer/scooter)
  Interactive find and replace in the terminal

- [sd](https://github.com/chmln/sd)
  Intuitive find-and-replace command, a `sed` alternative

- [sttr](https://github.com/abhimanyu003/sttr)
  Cross-platform CLI to perform various string operations

- [termscp](https://github.com/veeso/termscp)
  Feature-rich terminal UI file transfer and explorer (SCP/SFTP/FTP/S3)


- [trash-cli-rs](https://github.com/orf/trash)
  Safe `rm` replacement that moves files to the trash

- [xplr](https://github.com/sayanarijit/xplr)
  Hackable, minimal, fast TUI file manager with Lua scripting

- [yazi](https://github.com/sxyazi/yazi)
  Blazing fast terminal file manager with asynchronous I/O and a lot of features

## git

- [delta](https://github.com/dandavison/delta)
  Syntax-highlighting pager for git, diff, and grep output

- [difftastic](https://github.com/Wilfred/difftastic)
  Structural diff tool that understands code syntax

- [git-flow-next](https://github.com/gittower/git-flow-next)
  Modern reimplementation of git-flow in Go

- [gitleaks](https://github.com/gitleaks/gitleaks)
  Detect secrets and sensitive data in git repositories

- [lazygit](https://github.com/jesseduffield/lazygit)
  Simple terminal UI for `git` commands

- [mergiraf](https://codeberg.org/mergiraf/mergiraf)
  Syntax-aware merge driver for `git` - in many cases makes conflict resolution much easier

- [serie](https://github.com/lusingander/serie)
  Alternative to `git log --graph`. Renders pretty git commit graph in terminal -
and nothing else


- [worktrunk](https://github.com/max-sixty/worktrunk)
  Git worktree manager — fast branch switching without stashing

## HTTP

- [caddy](https://github.com/caddyserver/caddy)
  Fast, automatic HTTPS web server with TLS (pretender to the Nginx throne)

- [curlie](https://github.com/rs/curlie)
  curl frontend that adds HTTPie-style formatting and usability

- [hurl](https://github.com/Orange-OpenSource/hurl)
  Run and test HTTP requests defined in plain text

- [restish](https://github.com/rest-sh/restish)
  CLI for interacting with REST-ish HTTP APIs, `curl` alternative

- [xh](https://github.com/ducaale/xh)
  Friendly and fast HTTP client, `HTTPie` and `curl` alternative

## Logs

Utilities that allow easier browsing of logs in terminal. Most of them really need
that logs are in correct, strict data format. If you want something that just works
and tries very hard to parse every conceivable log format without you needing to
write any kind of template or regex, use `lnav`. To take absolute control on parsing
and converting logs use `vector`. Everything else here is pretty eye-candy that will
not work most of the time for most of your logs.

- [gonzo](https://github.com/control-theory/gonzo)
  A powerful, real-time log analysis TUI. Supports OTLP (OpenTelemetry) format
natively, with automatic detection of JSON, logfmt, and plain text logs.


- [hl](https://github.com/pamburus/hl)
  Fast log viewer for `JSON` or `logfmt` logs.

- [lazyjournal](https://github.com/Lifailon/lazyjournal)
  TUI for browsing systemd journal and Docker logs

- [lnav](https://github.com/tstack/lnav)
  Tries really hard to figure out structure in "not-so-structured" input data
(logs). Most of the time works like a magic, extracting maximum of structured
data from your logs without you needing to define single line of capture
template or regex. Aggregates everything into TUI and provides syntax
highlighting, filtering, and SQL querying


- [logdy](https://github.com/logdyhq/logdy-core)
  Aggregates log input and then starts (self-hosted, embedded) Web UI that can be used for viewing and filtering them. Multiple input modes (files, stdin, sockets, REST API)

- [loggo](https://github.com/aurc/loggo)
  Define one or more local or remote (SSH) log streams. Then see them all
aggregated to single TUI on your local machine. The only problem is that they
should really be in correctly structured JSON, for this utility to be useful.


- [nerdlog](https://github.com/dimonomid/nerdlog)
  SSH to (one or more) remote host(s), and live view one or more logs in TUI


- [rhit](https://github.com/canop/rhit)
  Reads nginx log files in their standard location, does some analysis and tells you about it in pretty tables in your console, storing and polluting nothing. Not so useful for viewing individual logs, but gives some useful metrics that are calculated from logs.

- [tailspin](https://github.com/bensadeh/tailspin)
  better `tail`: tries to colorizes any kind of input thrown at it


- [vector](https://github.com/vectordotdev/vector)
  Define input formats or use one of many pre-defined. Then apply them to logs, metrics and traces. Finally deliver structured data extracted from logs to many supported log sinks ... or to your terminal. Very similar in functionality to `Logstash`, but with much more power, many more features and much less runtime bloat (Rust instead of JDK).

## Networking

- [boring](https://github.com/alebeck/boring)
  The boring SSH tunnel manager. Uses local SSH config and keys to manage SSH tunnels in more human-friendly way.

- [dog](https://github.com/ogham/dog)
  A command-line DNS client (better `dig`). Unmaintained, see `doggo` instead

- [doggo](https://github.com/mr-karan/doggo)
  Command-line DNS client for humans, inspired by `dog` but written in Go

## Shell

Things to improve builtin shell capabilities.

- [atuin](https://github.com/atuinsh/atuin)
  Shell history search backed by SQLite with sync

- [carapace](https://github.com/carapace-sh/carapace-bin)
  Multi-shell completion generator for command-line tools

- [fzf](https://github.com/junegunn/fzf)
  General-purpose command-line fuzzy finder

- [skim](https://github.com/skim-rs/skim)
  Fuzzy finder in Rust (alternative to `fzf`)

- [starship](https://github.com/starship/starship)
  Minimal, blazing-fast, infinitely customizable shell prompt

- [vivid](https://github.com/sharkdp/vivid)
  `LS_COLORS` manager with multiple themes for colorful terminal output
(replacement for `dircolors`)


- [zoxide](https://github.com/ajeetdsouza/zoxide)
  Smarter `cd` command with frecency-based directory jumping

## System

- [bottom](https://github.com/ClementTsang/bottom)
  Graphical process/system monitor for the terminal (`htop`/`top` alternative)

- [btop](https://github.com/aristocratos/btop)
  A monitor of resources (`top` / `htop` alternative)

- [duf](https://github.com/muesli/duf)
  Disk Usage/Free Utility — human-friendly `df` alternative

- [dust](https://github.com/bootandy/dust)
  Intuitive disk usage visualizer, a `du` alternative

- [dysk](https://github.com/Canop/dysk)
  Terminal utility to get information on filesystems (`df` alternative)

- [erdtree](https://github.com/solidiquis/erdtree)
  Multi-threaded filesystem tree visualizer and disk usage analyzer, gitignore-aware

- [procs](https://github.com/dalance/procs)
  Modern replacement for ps with colored output and search

