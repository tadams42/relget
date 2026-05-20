use std::path::{Path, PathBuf};

pub fn uninstall_app(prefix: &Path, exe_name: &str) -> Vec<PathBuf> {
    let mut removed = Vec::new();

    try_remove(prefix.join("bin").join(exe_name), &mut removed);

    try_remove(
        prefix
            .join("share/zsh/site-functions")
            .join(format!("_{exe_name}")),
        &mut removed,
    );
    try_remove(
        prefix
            .join("share/bash-completion/completions")
            .join(exe_name),
        &mut removed,
    );
    try_remove(
        prefix
            .join("share/fish/vendor_completions.d")
            .join(format!("{exe_name}.fish")),
        &mut removed,
    );

    for section in 1u8..=9 {
        let man_dir = prefix.join(format!("share/man/man{section}"));
        if !man_dir.is_dir() {
            continue;
        }
        let Ok(entries) = std::fs::read_dir(&man_dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let filename = entry.file_name();
            let name = filename.to_string_lossy();
            if matches_man_page(&name, exe_name) {
                try_remove(entry.path(), &mut removed);
            }
        }
    }

    removed
}

fn matches_man_page(filename: &str, exe_name: &str) -> bool {
    let stem = filename.strip_suffix(".gz").unwrap_or(filename);

    // {exe_name}.{digits}
    if let Some(rest) = stem.strip_prefix(exe_name) {
        if let Some(section) = rest.strip_prefix('.') {
            if !section.is_empty() && section.chars().all(|c| c.is_ascii_digit()) {
                return true;
            }
        }
    }

    // {exe_name}-{anything}.{digits} or {exe_name}_{anything}.{digits}
    for sep in ['-', '_'] {
        let prefix = format!("{exe_name}{sep}");
        if let Some(rest) = stem.strip_prefix(prefix.as_str()) {
            if let Some(dot_pos) = rest.rfind('.') {
                let section = &rest[dot_pos + 1..];
                if !section.is_empty() && section.chars().all(|c| c.is_ascii_digit()) {
                    return true;
                }
            }
        }
    }

    false
}

fn try_remove(path: PathBuf, removed: &mut Vec<PathBuf>) {
    if std::fs::remove_file(&path).is_ok() {
        removed.push(path);
    }
}

#[cfg(test)]
mod tests {
    use super::matches_man_page;

    #[test]
    fn test_matches_man_page() {
        assert!(matches_man_page("rg.1", "rg"));
        assert!(matches_man_page("rg.1.gz", "rg"));
        assert!(matches_man_page("rg.5", "rg"));
        assert!(matches_man_page("caddy-adapt.1", "caddy"));
        assert!(matches_man_page("caddy-adapt.1.gz", "caddy"));
        assert!(matches_man_page("eza_colors.5", "eza"));
        assert!(matches_man_page("eza_colors.5.gz", "eza"));

        assert!(!matches_man_page("rg.txt", "rg"));
        assert!(!matches_man_page("rgb.1", "rg"));
        assert!(!matches_man_page("rg", "rg"));
        assert!(!matches_man_page("rg.", "rg"));
        assert!(!matches_man_page("rg.abc", "rg"));
    }
}
