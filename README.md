# binup

Downloads and installs bunch of cmdline utilities directly from `GitHub` and `Codeberg`
releases into `/usr/local`. Installs app itself, `man` pages and completion for `ZSH`,
`Bash` and `Fish` into standard locations.

**TL;DR**

```sh
curl -fsSL https://github.com/tadams42/binup/releases/latest/download/binup-x86_64-linux.tar.gz | tar xz

sudo ./binup --gh-token-source load --minimal-set

rm ./binup
```

## Why?

Whenever I need to `ssh` to some new VM, I usually loose access to my favorite
collection of CLI tools. Sometimes `sudo apt install ...` or similar can help. Often
times it can't.

`binup` always works ... though downloaded binaries may not 😎. The risk is acceptable
to me `99.999%` of times. `YMMV`.

Installing into `/usr/local` doesn't interfere with the rest of the system. Ie. you can
have `ripgrep` installed from both, official distro package and from `binup`: updating
any of them will not overwrite the other. Which one gets used when you call `ripgrep`
from your shell, depends on your `$PATH`. In most modern distros, stuff from
`/usr/local` has priority.

## Non goals and limitations

- `binup` is **NOT** a fully blown package manager
- there is no way to select the version of installed binary - `binup` **ALWAYS**
  installs latest version of each supported app:
  - which may not work on your current system (usually if your system is too old)
  - or that app's released binary is broken
  - or the binary works but the latest version that `binup` had just installed, is no
    longer compatible with whatever you have on your system that depends on it; and
    `binup` had just this moment installed it to location that makes sure this broken
    version is called by everything 😎
  - or ...
- there is no way to uninstall installed files (except by deleting manually from
  `/usr/local`)
- `binup` works on and installs utilities for **Linux only**; you may be able to make it
  work on some other systems, but it was never intended to be used like that [^1]
- `binup` downloads only `x86_64` binaries

## How to use it?

```sh
# install everything into /usr/local
sudo binup

# ... or install everything into ~/.local
binup --prefix ~/.local

# install a subset of apps
binup --prefix ~/.local --apps rg,bat,fzf

# install the hand-picked minimal set
binup --prefix ~/.local --minimal-set
```

`GitHub` applies rate limiting to unauthenticated API requests. Providing a token avoids
hitting those limits.

```sh
# prompt for GitHub token interactively
binup --gh-token-source prompt

# load GitHub token from GITHUB_API_TOKEN env var or ~/.config/github/api_token
binup --gh-token-source load

# load Codeberg token from CODEBERG_API_TOKEN env var or ~/.config/codeberg/api_token
binup --cb-token-source load

# prompt for Codeberg token interactively
binup --cb-token-source prompt
```

`binup` always uses `~/.cache/binup` for stuff downloaded from `GitHub` and `Codeberg`.

## Supported apps

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

[^1]: Previously, `binup` had been written in Python.
      Workflow that required me to deploy Python to be able to deploy `binup` to be able
      to deploy various CLI utilities was not one of my brightest ideas. Luckily, Claude
      was able to rewrite the whole thing in Rust so I was able to abandon that silly
      Python project. 😎
