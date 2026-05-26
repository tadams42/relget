use std::fmt;

#[derive(Debug)]
pub struct RateLimitError {
    pub site: &'static str,
}

impl fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} API rate limit exceeded", self.site)
    }
}

impl std::error::Error for RateLimitError {}
