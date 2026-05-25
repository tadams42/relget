use std::fs;
use std::path::{Path, PathBuf};

use relget::apps::all_app_entries;

fn main() {
    let task = std::env::args().nth(1);
    match task.as_deref() {
        Some("update-readme") => update_readme(),
        _ => {
            eprintln!("Available tasks:");
            eprintln!("  update-readme  Regenerate the Supported apps section in README.md");
        }
    }
}

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
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

fn update_readme() {
    let readme_path = project_root().join("README.md");

    let mut entries: Vec<_> = all_app_entries().iter().collect();
    entries.sort_by(|a, b| a.category.cmp(b.category).then_with(|| a.id.cmp(b.id)));

    let mut app_lines: Vec<String> = Vec::new();
    let mut current_category: Option<&str> = None;
    for entry in &entries {
        if current_category != Some(entry.category) {
            if current_category.is_some() {
                app_lines.push(String::new());
            }
            app_lines.push(format!("### {}", category_display_name(entry.category)));
            app_lines.push(String::new());
            current_category = Some(entry.category);
        }
        app_lines.push(format!("- [{}]({}) — {}", entry.id, entry.url, entry.description));
    }

    let readme = fs::read_to_string(&readme_path).expect("failed to read README.md");
    let lines: Vec<&str> = readme.lines().collect();

    let idx_start = lines
        .iter()
        .position(|l| *l == "## Supported apps")
        .expect("'## Supported apps' marker not found in README.md");
    let idx_end = lines
        .iter()
        .position(|l| l.starts_with("[^1]: "))
        .expect("'[^1]:' marker not found in README.md");

    let mut new_lines: Vec<String> =
        lines[..=idx_start].iter().map(|l| l.to_string()).collect();
    new_lines.push(String::new());
    new_lines.extend(app_lines);
    new_lines.push(String::new());
    new_lines.extend(lines[idx_end..].iter().map(|l| l.to_string()));

    fs::write(&readme_path, new_lines.join("\n") + "\n").expect("failed to write README.md");
    println!("README.md updated.");
}
