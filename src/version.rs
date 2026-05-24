use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AppVersion(pub u64, pub u64, pub u64);

impl AppVersion {
    /// Scan `s` for the first occurrence of a `major.minor.patch` pattern and parse it.
    pub fn find_in(s: &str) -> Option<Self> {
        let b = s.as_bytes();
        let mut i = 0;
        while i < b.len() {
            if !b[i].is_ascii_digit() {
                i += 1;
                continue;
            }
            let start = i;
            while i < b.len() && b[i].is_ascii_digit() {
                i += 1;
            }
            if i >= b.len() || b[i] != b'.' {
                continue;
            }
            i += 1;
            let minor_start = i;
            while i < b.len() && b[i].is_ascii_digit() {
                i += 1;
            }
            if i == minor_start || i >= b.len() || b[i] != b'.' {
                i = start + 1;
                continue;
            }
            i += 1;
            let patch_start = i;
            while i < b.len() && b[i].is_ascii_digit() {
                i += 1;
            }
            if i == patch_start {
                i = start + 1;
                continue;
            }
            if let Some(v) = Self::parse(&s[start..i]) {
                return Some(v);
            }
            i = start + 1;
        }
        None
    }

    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        // Strip leading non-numeric prefix (e.g. 'v', 'V', letters)
        let s = s.trim_start_matches(|c: char| !c.is_ascii_digit());
        if s.is_empty() {
            return None;
        }
        // Replace hyphens with dots for versions like "1.2.3-4"
        let s = s.replace('-', ".");
        let parts: Vec<u64> = s
            .split('.')
            .take(3)
            .map(|p| {
                let num: String = p.chars().take_while(|c| c.is_ascii_digit()).collect();
                num.parse::<u64>().unwrap_or(0)
            })
            .collect();

        if parts.is_empty() {
            return None;
        }

        Some(AppVersion(
            *parts.first().unwrap_or(&0),
            *parts.get(1).unwrap_or(&0),
            *parts.get(2).unwrap_or(&0),
        ))
    }
}

impl fmt::Display for AppVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}
