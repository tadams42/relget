use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=src/apps/mod.rs");

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_dir = Path::new(&manifest_dir);

    update_readme(manifest_dir);
}

fn update_readme(manifest_dir: &Path) {
    let mod_path = manifest_dir.join("src/apps/mod.rs");
    let readme_path = manifest_dir.join("README.md");

    let mod_content = fs::read_to_string(&mod_path).expect("Failed to read src/apps/mod.rs");

    let mut entries: Vec<(String, String)> = parse_app_entries(&mod_content);
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let app_lines: Vec<String> = entries
        .iter()
        .map(|(id, url)| format!("- [{}]({})", id, url))
        .collect();

    let readme = fs::read_to_string(&readme_path).expect("Failed to read README.md");
    let lines: Vec<&str> = readme.lines().collect();

    let idx_first = lines
        .iter()
        .position(|l| *l == "## Supported apps")
        .expect("'Supported apps' marker not found in README.md");
    let idx_last = lines
        .iter()
        .position(|l| l.starts_with("[^1]: "))
        .expect("'[^1]:' marker not found in README.md");

    let mut new_lines: Vec<String> = lines[..=idx_first].iter().map(|l| l.to_string()).collect();
    new_lines.push(String::new());
    new_lines.extend(app_lines);
    new_lines.push(String::new());
    new_lines.extend(lines[idx_last..].iter().map(|l| l.to_string()));

    let new_content = new_lines.join("\n") + "\n";
    fs::write(&readme_path, new_content).expect("Failed to write README.md");
}

fn parse_app_entries(content: &str) -> Vec<(String, String)> {
    let mut entries = Vec::new();
    let mut pending: Option<(Option<String>, Option<String>)> = None;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("AppEntry {") {
            let id = extract_quoted_field(line, "id:");
            let url = extract_quoted_field(line, "url:");
            if let (Some(id), Some(url)) = (id.clone(), url.clone()) {
                entries.push((id, url));
            } else {
                pending = Some((id, url));
            }
        } else if let Some((ref mut id, ref mut url)) = pending {
            if id.is_none() {
                *id = extract_quoted_field(line, "id:");
            }
            if url.is_none() {
                *url = extract_quoted_field(line, "url:");
            }
            if id.is_some() && url.is_some() {
                entries.push((id.take().unwrap(), url.take().unwrap()));
                pending = None;
            }
        }
    }
    entries
}

fn extract_quoted_field(line: &str, field: &str) -> Option<String> {
    let start = line.find(field)? + field.len();
    let rest = line[start..].trim_start().strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}
