use std::fs;
use std::path::{Path, PathBuf};

use relget::all_app_entries;

fn main() {
    let task = std::env::args().nth(1);
    match task.as_deref() {
        Some("update-docs") => {
            update_supported_apps();
            update_changelog();
        }
        _ => {
            eprintln!("Available tasks:");
            eprintln!(
                "  update-docs  Regenerate SUPPORTED_APPS.md and ## unreleased changelog section"
            );
        }
    }
}

const NOISE_PREFIXES: &[&str] = &[
    "build:", "ci:", "docs:", "chore:", "refact:", "refactor", "test",
];

fn update_changelog() {
    let tag_output = std::process::Command::new("git")
        .args(["describe", "--tags", "--abbrev=0"])
        .current_dir(project_root())
        .output()
        .expect("failed to run git describe");

    if !tag_output.status.success() {
        eprintln!("git describe failed — no tags found?");
        std::process::exit(1);
    }

    let tag = String::from_utf8(tag_output.stdout)
        .expect("git describe output is not UTF-8")
        .trim()
        .to_string();

    let log_output = std::process::Command::new("git")
        .args(["log", &format!("{}..HEAD", tag), "--format=%s"])
        .current_dir(project_root())
        .output()
        .expect("failed to run git log");

    if !log_output.status.success() {
        eprintln!("git log failed");
        std::process::exit(1);
    }

    let log_text = String::from_utf8(log_output.stdout).expect("git log output is not UTF-8");

    let bullet_lines: Vec<String> = log_text
        .lines()
        .filter(|l| !l.is_empty())
        .filter(|l| !NOISE_PREFIXES.iter().any(|p| l.starts_with(p)))
        .map(|l| format!("- {}", l))
        .collect();

    let changelog_path = project_root().join("CHANGELOG.md");
    let changelog = fs::read_to_string(&changelog_path).expect("failed to read CHANGELOG.md");
    let lines: Vec<&str> = changelog.lines().collect();

    let idx_start = lines
        .iter()
        .position(|l| *l == "## unreleased")
        .expect("'## unreleased' heading not found in CHANGELOG.md");

    let idx_end = lines[idx_start + 1..]
        .iter()
        .position(|l| l.starts_with("## "))
        .map(|rel| idx_start + 1 + rel)
        .unwrap_or(lines.len());

    let mut new_lines: Vec<String> = lines[..=idx_start].iter().map(|l| l.to_string()).collect();
    new_lines.push(String::new());
    new_lines.extend(bullet_lines);
    new_lines.push(String::new());
    new_lines.extend(lines[idx_end..].iter().map(|l| l.to_string()));

    fs::write(&changelog_path, new_lines.join("\n") + "\n").expect("failed to write CHANGELOG.md");
    println!("CHANGELOG.md updated (commits since {}).", tag);
}

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn category_display_name(category: &str) -> &str {
    match category {
        "containers" => "Containers",
        "data_processing" => "Data - processing JSON/YAML/CSV",
        "dev_envs" => "Development environments",
        "coding" => "Coding",
        "databases" => "Databases",
        "docs_diag" => "Documentation and diagrams",
        "encryption" => "Encryption and secrets management",
        "files" => "Files",
        "git" => "git",
        "http" => "HTTP",
        "logs" => "Logs",
        "system" => "System",
        "shell" => "Shell",
        "music" => "Music",
        other => other,
    }
}

fn update_supported_apps() {
    let path = project_root().join("SUPPORTED_APPS.md");

    let mut entries: Vec<_> = all_app_entries().iter().collect();
    entries.sort_by(|a, b| a.category.cmp(&b.category).then_with(|| a.id.cmp(&b.id)));

    let mut lines: Vec<String> = vec!["# Supported apps".into(), String::new()];
    let mut current_category: Option<&str> = None;

    for entry in &entries {
        if current_category != Some(entry.category.as_str()) {
            if current_category.is_some() {
                lines.push(String::new());
            }
            lines.push(format!("## {}", category_display_name(&entry.category)));
            lines.push(String::new());
            current_category = Some(entry.category.as_str());
        } else {
            lines.push(String::new());
        }
        lines.push(format!("- [{}]({})", entry.id, entry.url));
        lines.push(format!("  {}", entry.description));
    }
    lines.push(String::new());

    fs::write(&path, lines.join("\n") + "\n").expect("failed to write SUPPORTED_APPS.md");
    println!("SUPPORTED_APPS.md written.");
}
