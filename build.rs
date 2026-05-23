use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=src/apps/mod.rs");

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_dir = Path::new(&manifest_dir);

    update_readme(manifest_dir);
}

fn category_display_name(category: &str) -> &str {
    match category {
        "containers" => "Containers",
        "data" => "Data",
        "dev_envs" => "Dev envs",
        "dev_tools" => "Dev tools",
        "files" => "Files",
        "git" => "Git",
        "http" => "HTTP",
        "logs" => "Logs",
        "other" => "Other",
        "shell" => "Shell",
        other => other,
    }
}

fn update_readme(manifest_dir: &Path) {
    let mod_path = manifest_dir.join("src/apps/mod.rs");
    let readme_path = manifest_dir.join("README.md");

    let mod_content = fs::read_to_string(&mod_path).expect("Failed to read src/apps/mod.rs");

    let mut entries: Vec<(String, String, String)> = parse_app_entries(&mod_content);

    entries.sort_by(|a, b| a.2.cmp(&b.2).then_with(|| a.0.cmp(&b.0)));

    let mut app_lines: Vec<String> = Vec::new();
    let mut current_category: Option<&str> = None;
    for (id, url, cat) in &entries {
        let cat_str = cat.as_str();
        if current_category != Some(cat_str) {
            if current_category.is_some() {
                app_lines.push(String::new());
            }
            app_lines.push(format!("### {}", category_display_name(cat_str)));
            app_lines.push(String::new());
            current_category = Some(cat_str);
        }
        app_lines.push(format!("- [{}]({})", id, url));
    }

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

fn parse_app_entries(content: &str) -> Vec<(String, String, String)> {
    let mut entries = Vec::new();
    let mut pending: Option<(Option<String>, Option<String>, Option<String>)> = None;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("AppEntry {") {
            let id = extract_quoted_field(line, "id:");
            let url = extract_quoted_field(line, "url:");
            let cat = extract_quoted_field(line, "category:");
            if let (Some(id), Some(url), Some(cat)) = (id.clone(), url.clone(), cat.clone()) {
                entries.push((id, url, cat));
            } else {
                pending = Some((id, url, cat));
            }
        } else if let Some((ref mut id, ref mut url, ref mut cat)) = pending {
            if id.is_none() {
                *id = extract_quoted_field(line, "id:");
            }
            if url.is_none() {
                *url = extract_quoted_field(line, "url:");
            }
            if cat.is_none() {
                *cat = extract_quoted_field(line, "category:");
            }
            if id.is_some() && url.is_some() && cat.is_some() {
                entries.push((id.take().unwrap(), url.take().unwrap(), cat.take().unwrap()));
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
