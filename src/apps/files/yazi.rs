// - [yazi](https://github.com/sxyazi/yazi)
//   - Rust, async I/O
//   - plugins and package manager for them
//   - images, code highlighting, multi tab, cross-directory selection, scrollable Preview (for
//     videos, PDFs, archives, code, directories, etc.)
//   - bulk rename/vreate, archive Extraction, visual mode, file chooser, git integration, mount
//     manager
//   - optional dependencies:
//     - `file`
//     - `ffmpeg`
//     - [poppler](https://poppler.freedesktop.org/) - for PDF preview
//     - `fd`
//     - `rg`
//     - `fzf`
//     - `zoxide`
//     - [resvg](https://github.com/linebender/resvg) - Rust cli for SVG preview
//     - `ImageMagick`
//     - `xclip` / `wl-clipboard` / `xsel` for Linux clipboard support
//     - [Überzug++](https://github.com/jstkdng/ueberzugpp)
//       - command line utility written in C++ which allows to draw images on terminals by using
//         X11/wayland child windows, sixels, kitty and iterm2...
//       - needs [PPA](https://software.opensuse.org/download.html?project=home%3Ajustkidding&package=ueberzugpp)
//         to install on Ubuntu
//     - [chafa](https://hpjansson.org/chafa/) - fallback image preview in terminal, renders images
//       as ASCII art (Unicode block)
//
// - [xplr](https://github.com/sayanarijit/xplr)
//   - Rust
//   - hackable, minimal, fast
//   - provides almost nothing out of the box, but is infinitelly configurable with Lua
//
// - [nnn](https://github.com/jarun/nnn)
//   - `n³`
//   - written in C, but has statically linked `musl` builds available
//   - huge repository of plugins (shell scripts or compiled binaries - no embeded interpreter ie.
//     Lua)
//
// - [rfm](https://github.com/dsxmachina/rfm)
//   - Rust
//   - terminal file manager with VI-bindings
//   - heavily inspired by ranger, but to make it fast
//   - basic preview engine, shell operations, etc..
//
// - [joshuto](https://github.com/kamiyaa/joshuto)
//   - ranger-like terminal file manager written in Rust
//
// - [clifm](https://github.com/leo-arch/clifm)
//   - C
//   - bookmarks, search, file selection, file tags, file filters, file previews (including image
//     previews), bulk rename, archiving, trash, file opener, directory jumper, autocommands,
//     workspaces, plugins, autosuggestions...
//   - very thing TUI, tries to behave more like CLI - you type commands in it instead opening
//     menues: "No GUI, no TUI, and no menus: just you and a powerful, file-management-oriented
//     command line."
//   - not to be confused with `https://github.com/pasqu4le/clifm` that is also terminal file
//     manager, written in Haskell and unmaintained
//
// - [felix](https://github.com/kyoheiu/felix)
//   - Rust
//   - very simple, no plugins, no extensive configurability
//   - must have `chafa` installed for image preview
//
// - [lf](https://github.com/gokcehan/lf)
//   - written in `Go`
//   - heavily inspired by `ranger`
//   - most functionality is not built-in but instead relies on shell and external utilities (with
//     many examples in `integrations` docs)
//   - doesn't support tabs or windows (relies on terminal multiplexers instead)
//
// - [hunter](https://github.com/rabite0/hunter)
//   - Rust
//   - `ranger` inspired, Emacs-flavoured
//   - tabs, bookmarks, search, filter, preview
//   - It has no built in primitives for file manipulation like delete, rename, move, and so on.
//     Instead it relies on its easy and extensive integration with the standard cli tools to do its
//     job
//   - seems unmaintained
//
// - [broot](https://github.com/Canop/broot)
//   - Rust
//   - autmatically hides `.gitignre`-ed files
//   - very simple interface - tree of files, directories and sizs
//   - fast (fuzzy) search and selection
//
// - [vifm](https://github.com/vifm/vifm)
//   - written in C
//   - curses based, very Vim-like
//
// - [mc](https://github.com/MidnightCommander/mc)
//   - written in C, curses based
//   - old, reliable
//
// - [ranger](https://github.com/ranger/ranger)
//   - written in Python

use std::sync::Arc;

use anyhow::Result;

use crate::{
    App, AppAssets, AppBinary, AppVersion, ArchiveExtractor, Completion, RelgetClient, Shell,
};

pub struct Yazi {
    client: Arc<dyn RelgetClient>,
}

impl Yazi {
    pub const ID: &'static str = "yazi";
    const OWNER: &'static str = "sxyazi";
    const REPO: &'static str = "yazi";
    const EXE_NAME: &'static str = "yazi";
    pub fn new(client: Arc<dyn RelgetClient>) -> Self { Self { client } }
}

impl App for Yazi {
    fn exe_name(&self) -> &str { Self::EXE_NAME }

    fn released_version(&self) -> Result<AppVersion> {
        self.client
            .latest_release(Self::OWNER, Self::REPO)?
            .version()
    }

    fn assets(&self) -> AppAssets {
        AppAssets {
            binary: Some(AppBinary::new(Self::EXE_NAME)),
            other_bins: vec![AppBinary::new("ya")],
            completions: vec![
                Completion::new(Shell::Zsh, Self::EXE_NAME),
                Completion::new(Shell::Bash, Self::EXE_NAME),
                Completion::new(Shell::Fish, Self::EXE_NAME),
                Completion::new(Shell::Zsh, "ya"),
                Completion::new(Shell::Bash, "ya"),
                Completion::new(Shell::Fish, "ya"),
            ],
            ..Default::default()
        }
    }

    fn download(&self) -> Result<AppAssets> {
        let release = self.client.latest_release(Self::OWNER, Self::REPO)?;
        let name = release.find_asset(|a| a == "yazi-x86_64-unknown-linux-musl.zip")?;
        let asset = self.client.download_asset(Self::OWNER, Self::REPO, &name)?;
        let e = ArchiveExtractor::new(&name, asset.data);
        Ok(AppAssets {
            binary: Some(AppBinary::new_with_data("yazi", e.extract_by_filename("yazi")?)),
            other_bins: vec![AppBinary::new_with_data("ya", e.extract_by_filename("ya")?)],
            completions: vec![
                Completion::new_with_data(Shell::Zsh, "yazi", e.extract_by_filename("_yazi")?),
                Completion::new_with_data(Shell::Bash, "yazi", e.extract_by_filename("yazi.bash")?),
                Completion::new_with_data(Shell::Fish, "yazi", e.extract_by_filename("yazi.fish")?),
                Completion::new_with_data(Shell::Zsh, "ya", e.extract_by_filename("_ya")?),
                Completion::new_with_data(Shell::Bash, "ya", e.extract_by_filename("ya.bash")?),
                Completion::new_with_data(Shell::Fish, "ya", e.extract_by_filename("ya.fish")?),
            ],
            ..Default::default()
        })
    }
}
