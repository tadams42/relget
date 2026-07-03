use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
        // Components parse up to their first non-digit ("3rc1" → 3). A component with no
        // leading digit at all ends the parse: "1.x.3" is (1, 0, 0), not (1, 0, 3).
        let mut nums = [0u64; 3];
        for (slot, part) in nums.iter_mut().zip(s.split('.')) {
            let digits: String = part.chars().take_while(char::is_ascii_digit).collect();
            let Ok(num) = digits.parse::<u64>() else {
                break;
            };
            *slot = num;
        }
        Some(AppVersion(nums[0], nums[1], nums[2]))
    }
}

impl fmt::Display for AppVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple() {
        assert_eq!(AppVersion::parse("1.2.3"), Some(AppVersion(1, 2, 3)));
    }

    #[test]
    fn parse_with_v_prefix() {
        assert_eq!(AppVersion::parse("v1.2.3"), Some(AppVersion(1, 2, 3)));
    }

    #[test]
    fn parse_hyphen_becomes_dot_only_first_three_components() {
        assert_eq!(AppVersion::parse("1.2.3-4"), Some(AppVersion(1, 2, 3)));
    }

    #[test]
    fn parse_missing_patch_defaults_to_zero() {
        assert_eq!(AppVersion::parse("1.0"), Some(AppVersion(1, 0, 0)));
    }

    #[test]
    fn parse_major_only_defaults_rest_to_zero() {
        assert_eq!(AppVersion::parse("1"), Some(AppVersion(1, 0, 0)));
    }

    #[test]
    fn parse_empty_returns_none() {
        assert_eq!(AppVersion::parse(""), None);
    }

    #[test]
    fn parse_stops_at_non_numeric_component() {
        // "1.x.3" must not silently become 1.0.3
        assert_eq!(AppVersion::parse("1.x.3"), Some(AppVersion(1, 0, 0)));
    }

    #[test]
    fn parse_component_trailing_junk_is_trimmed() {
        assert_eq!(AppVersion::parse("1.2.3rc1"), Some(AppVersion(1, 2, 3)));
    }

    #[test]
    fn parse_prerelease_suffix_defaults_patch_to_zero() {
        assert_eq!(AppVersion::parse("1.2-rc1"), Some(AppVersion(1, 2, 0)));
    }

    #[test]
    fn parse_all_letters_returns_none() {
        assert_eq!(AppVersion::parse("abc"), None);
    }

    #[test]
    fn ordering_patch_is_numeric_not_lexicographic() {
        assert!(AppVersion(1, 2, 10) > AppVersion(1, 2, 9));
    }

    #[test]
    fn ordering_major_wins() {
        assert!(AppVersion(2, 0, 0) > AppVersion(1, 9, 9));
    }

    #[test]
    fn find_in_skips_arch_numbers() {
        // Ensures "x86_64" does not corrupt extraction; "14.1.0" should win.
        assert_eq!(
            AppVersion::find_in("ripgrep-14.1.0-x86_64-unknown-linux-musl"),
            Some(AppVersion(14, 1, 0))
        );
    }

    #[test]
    fn find_in_plain_version_string() {
        assert_eq!(
            AppVersion::find_in("starship v1.21.1 x86_64-unknown-linux-musl"),
            Some(AppVersion(1, 21, 1))
        );
    }

    #[test]
    fn find_in_no_version_returns_none() {
        assert_eq!(AppVersion::find_in("no version here"), None);
    }

    #[test]
    fn find_in_empty_returns_none() {
        assert_eq!(AppVersion::find_in(""), None);
    }
}
