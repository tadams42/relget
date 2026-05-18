use std::fs;
use std::path::Path;

#[path = "src/cli.rs"]
mod cli;

fn main() {
    println!("cargo:rerun-if-changed=src/apps/mod.rs");
    println!("cargo:rerun-if-changed=src/cli.rs");

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_dir = Path::new(&manifest_dir);

    update_readme(manifest_dir);
    generate_man_page(manifest_dir).expect("failed to generate man page");
}

fn update_readme(manifest_dir: &Path) {
    let mod_path = manifest_dir.join("src/apps/mod.rs");
    let readme_path = manifest_dir.join("README.md");

    let mod_content = fs::read_to_string(&mod_path).expect("Failed to read src/apps/mod.rs");

    let mut entries: Vec<(String, String)> = mod_content
        .lines()
        .filter_map(parse_app_entry)
        .collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let app_lines: Vec<String> = entries
        .iter()
        .map(|(id, url)| format!("- [{}]({})", id, url))
        .collect();

    let readme = fs::read_to_string(&readme_path).expect("Failed to read README.md");
    let lines: Vec<&str> = readme.lines().collect();

    let idx_first = lines
        .iter()
        .position(|l| *l == "Supported apps:")
        .expect("'Supported apps:' marker not found in README.md");
    let idx_last = lines
        .iter()
        .position(|l| *l == "## How to use it?")
        .expect("'## How to use it?' marker not found in README.md");

    let mut new_lines: Vec<String> =
        lines[..=idx_first].iter().map(|l| l.to_string()).collect();
    new_lines.push(String::new());
    new_lines.extend(app_lines);
    new_lines.push(String::new());
    new_lines.extend(lines[idx_last..].iter().map(|l| l.to_string()));

    let new_content = new_lines.join("\n") + "\n";
    fs::write(&readme_path, new_content).expect("Failed to write README.md");
}

fn generate_man_page(manifest_dir: &Path) -> std::io::Result<()> {
    use clap::CommandFactory;
    use clap_mangen::Man;
    use roff::{bold, roman, Roff};

    let cmd = cli::Cli::command();
    let man = Man::new(cmd);

    let out_path = manifest_dir.join("man").join("binup.1");
    fs::create_dir_all(out_path.parent().unwrap())?;

    let mut buf = Vec::<u8>::new();

    man.render_title(&mut buf)?;
    man.render_name_section(&mut buf)?;
    man.render_synopsis_section(&mut buf)?;
    man.render_description_section(&mut buf)?;

    let mut roff = Roff::new();
    roff.control("PP", std::iter::empty::<&str>())
        .text(vec![roman(
            "Installing into /usr/local does not interfere with packages managed by the \
             system package manager. Both can coexist; which one takes precedence depends on \
             $PATH ordering. In most Linux distributions /usr/local takes priority over /usr.",
        )])
        .control("PP", std::iter::empty::<&str>())
        .text(vec![roman("Only Linux on x86_64 is supported.")])
        .control("PP", std::iter::empty::<&str>())
        .text(vec![roman(
            "Shell completions can be generated for ZSH, Bash, and Fish.",
        )]);
    roff.to_writer(&mut buf)?;

    man.render_options_section(&mut buf)?;
    man.render_subcommands_section(&mut buf)?;

    let mut roff = Roff::new();
    roff.control("SH", ["EXAMPLES"])
        .control("PP", std::iter::empty::<&str>())
        .text(vec![roman(
            "Install or update all apps (requires write access to /usr/local):",
        )])
        .control("RS", ["4"])
        .control("nf", std::iter::empty::<&str>())
        .text(vec![bold("binup")])
        .control("fi", std::iter::empty::<&str>())
        .control("RE", std::iter::empty::<&str>())
        .control("PP", std::iter::empty::<&str>())
        .text(vec![roman("Install a subset of apps:")])
        .control("RS", ["4"])
        .control("nf", std::iter::empty::<&str>())
        .text(vec![
            bold("binup"),
            roman(" --apps rg --apps bat --apps fzf"),
        ])
        .control("fi", std::iter::empty::<&str>())
        .control("RE", std::iter::empty::<&str>())
        .control("PP", std::iter::empty::<&str>())
        .text(vec![roman("Install the predefined minimal set:")])
        .control("RS", ["4"])
        .control("nf", std::iter::empty::<&str>())
        .text(vec![bold("binup"), roman(" --minimal-set")])
        .control("fi", std::iter::empty::<&str>())
        .control("RE", std::iter::empty::<&str>())
        .control("PP", std::iter::empty::<&str>())
        .text(vec![roman(
            "Install into a user-local prefix (no root required):",
        )])
        .control("RS", ["4"])
        .control("nf", std::iter::empty::<&str>())
        .text(vec![bold("binup"), roman(" --prefix ~/.local")])
        .control("fi", std::iter::empty::<&str>())
        .control("RE", std::iter::empty::<&str>())
        .control("PP", std::iter::empty::<&str>())
        .text(vec![roman("List all supported app identifiers:")])
        .control("RS", ["4"])
        .control("nf", std::iter::empty::<&str>())
        .text(vec![bold("binup"), roman(" list-apps-ids")])
        .control("fi", std::iter::empty::<&str>())
        .control("RE", std::iter::empty::<&str>())
        .control("PP", std::iter::empty::<&str>())
        .text(vec![roman("Generate ZSH completions:")])
        .control("RS", ["4"])
        .control("nf", std::iter::empty::<&str>())
        .text(vec![bold("binup"), roman(" completions zsh")])
        .control("fi", std::iter::empty::<&str>())
        .control("RE", std::iter::empty::<&str>());
    roff.to_writer(&mut buf)?;

    let mut roff = Roff::new();
    roff.control("SH", ["API TOKENS"])
        .control("PP", std::iter::empty::<&str>())
        .text(vec![roman(
            "GitHub applies rate limiting to unauthenticated API requests. \
             Providing a token avoids hitting those limits.",
        )])
        .control("PP", std::iter::empty::<&str>())
        .text(vec![roman("GitHub token sources ("), bold("--gh-token-source"), roman("):")])
        .control("TP", std::iter::empty::<&str>())
        .text(vec![bold("prompt")])
        .text(vec![roman("Read the token from stdin (default).")])
        .control("TP", std::iter::empty::<&str>())
        .text(vec![bold("load")])
        .text(vec![roman(
            "Read from the GITHUB_API_TOKEN environment variable, \
             falling back to ~/.config/github/api_token.",
        )])
        .control("PP", std::iter::empty::<&str>())
        .text(vec![roman("Codeberg token sources ("), bold("--cb-token-source"), roman("):")])
        .control("TP", std::iter::empty::<&str>())
        .text(vec![bold("load")])
        .text(vec![roman(
            "Read from the CODEBERG_API_TOKEN environment variable, \
             falling back to ~/.config/codeberg/api_token (default).",
        )])
        .control("TP", std::iter::empty::<&str>())
        .text(vec![bold("prompt")])
        .text(vec![roman("Read the token from stdin.")]);
    roff.to_writer(&mut buf)?;

    let mut roff = Roff::new();
    roff.control("SH", ["CACHE"])
        .control("PP", std::iter::empty::<&str>())
        .text(vec![roman(
            "Downloaded data is cached under ~/.cache/binup/. \
             Release metadata has a one-hour TTL. \
             Downloaded assets are cached permanently, keyed by asset ID; \
             the cache is invalidated automatically when a new release is published.",
        )]);
    roff.to_writer(&mut buf)?;

    man.render_version_section(&mut buf)?;

    fs::write(&out_path, buf)
}

fn parse_app_entry(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    if !line.starts_with("AppEntry {") {
        return None;
    }
    let id = extract_quoted_field(line, "id:")?;
    let url = extract_quoted_field(line, "url:")?;
    Some((id, url))
}

fn extract_quoted_field(line: &str, field: &str) -> Option<String> {
    let start = line.find(field)? + field.len();
    let rest = line[start..].trim_start().strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}
