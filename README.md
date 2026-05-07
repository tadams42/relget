# rutils-downloader

Installs bunch of cmdline utilities into `/usr/local` directly from GitHub and Codeberg releases.

Installing into `/usr/local` doesn't interfere with the rest of the system. Ie. you can
have `ripgrep` installed from both, official distro package and this script and updating
any of them will not overwrite the other. Which one gets used when you call `ripgrep`
from your shell, depends on your `$PATH`. In most modern distros, stuff from
`/usr/local` has priority.

Supported operating systems:

- any and only Linux
- only `x86_64` architecture

Supported shells (for completions):

- ZSH
- Bash
- Fish

Supported apps:

- [aqua](https://github.com/aquaproj/aqua)
- [ast-grep](https://github.com/ast-grep/ast-grep)
- [atuin](https://github.com/atuinsh/atuin)
- [bat](https://github.com/sharkdp/bat)
- [caddy](https://github.com/caddyserver/caddy)
- [carapace](https://github.com/carapace-sh/carapace-bin)
- [chezmoi](https://github.com/twpayne/chezmoi)
- [d4s](https://github.com/jr-k/d4s)
- [dasel](https://github.com/TomWright/dasel)
- [delta](https://github.com/dandavison/delta)
- [difft](https://github.com/Wilfred/difftastic)
- [dockmate](https://github.com/shubh-io/DockMate)
- [dry](https://github.com/moncho/dry)
- [eza](https://github.com/eza-community/eza)
- [fd](https://github.com/sharkdp/fd)
- [fnm](https://github.com/Schniz/fnm)
- [fx](https://github.com/antonmedv/fx)
- [fzf](https://github.com/junegunn/fzf)
- [gitleaks](https://github.com/gitleaks/gitleaks)
- [go](https://go.dev/)
- [gojq](https://github.com/itchyny/gojq)
- [gonzo](https://github.com/control-theory/gonzo)
- [jid](https://github.com/simeji/jid)
- [jq](https://github.com/jqlang/jq)
- [jqp](https://github.com/noahgorstein/jqp)
- [lazydocker](https://github.com/jesseduffield/lazydocker)
- [lazygit](https://github.com/jesseduffield/lazygit)
- [lazyjournal](https://github.com/Lifailon/lazyjournal)
- [mdbook](https://github.com/rust-lang/mdBook)
- [mergiraf](https://codeberg.org/mergiraf/mergiraf)
- [mise](https://github.com/jdx/mise)
- [neovide](https://github.com/neovide/neovide)
- [rclone](https://github.com/rclone/rclone)
- [restish](https://github.com/rest-sh/restish)
- [rg](https://github.com/BurntSushi/ripgrep)
- [rust-analyzer](https://github.com/rust-lang/rust-analyzer)
- [sd](https://github.com/chmln/sd)
- [sk](https://github.com/skim-rs/skim)
- [starship](https://github.com/starship/starship)
- [stylua](https://github.com/JohnnyMorganz/stylua)
- [uv](https://github.com/astral-sh/uv)
- [xq](https://github.com/sibprogrammer/xq)
- [yq](https://github.com/mikefarah/yq)
- [zoxide](https://github.com/ajeetdsouza/zoxide)

## How to use it?

Build (needs Rust toolchain):

```sh
cargo build --release
```

Install or update apps (needs to be run as `root` to write into `/usr/local`):

```sh
sudo su -

# copy the binary somewhere on PATH
cp target/release/rutils-downloader /usr/local/bin/

# install everything
rutils-downloader

# install a subset
rutils-downloader --apps rg --apps bat --apps fzf

# install the hand-picked minimal set
rutils-downloader --minimal-set

# install into a different prefix (no sudo needed)
rutils-downloader --prefix ~/.local

# list all supported app identifiers
rutils-downloader list-apps-ids
```

## API tokens

GitHub applies rate limiting to unauthenticated API requests. Providing a token avoids hitting those limits.

```sh
# prompt for GitHub token interactively (default)
rutils-downloader --gh-token-source prompt

# load GitHub token from GITHUB_API_TOKEN env var or ~/.config/github/api_token
rutils-downloader --gh-token-source load

# load Codeberg token from CODEBERG_API_TOKEN env var or ~/.config/codeberg/api_token (default)
rutils-downloader --cb-token-source load

# prompt for Codeberg token interactively
rutils-downloader --cb-token-source prompt
```

## Other side-effects

- uses `~/.cache/rutils-downloader` for stuff downloaded from GitHub and Codeberg
