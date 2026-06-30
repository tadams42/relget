use anyhow::{Result, anyhow};
use chrono::{DateTime, Duration, Utc};

use registry_core::AppEntry;
use crate::{CodebergClient, Config, GithubClient, GitlabClient, ReleaseMetadata, RelgetClient};

pub(super) fn doctor(apps: &[AppEntry], offline: bool) -> Result<()> {
    let (gh_token, cb_token, gl_token) = if offline {
        (None, None, None)
    } else {
        (
            load_token("github", Config::github_token()?)?,
            load_token("codeberg", Config::codeberg_token()?)?,
            load_token("gitlab", Config::gitlab_token()?)?,
        )
    };

    let mut flagged: Vec<FlaggedApp> = Vec::new();
    let threshold = Utc::now() - Duration::days(365);

    for app in apps {
        let release = match fetch_release(
            &app.url,
            gh_token.clone(),
            cb_token.clone(),
            gl_token.clone(),
            offline,
        ) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("warning: {}: {}", app.id, e);
                continue;
            }
        };

        let published_at = release_date(&release);
        let version_str = release
            .version()
            .map(|v| v.to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        let date_str = published_at
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let asset_names = release.asset_names();
        let release_musl = release_has_x86_musl(&asset_names);

        let mut flags: Vec<DoctorFlag> = Vec::new();

        if let Some(date) = published_at {
            if date < threshold {
                flags.push(DoctorFlag::PotentiallyUnmaintained);
            }
        }

        if !app.has_declared_musl() && release_musl {
            flags.push(DoctorFlag::MuslNowAvailable);
        }
        if app.has_declared_musl() && !release_musl {
            flags.push(DoctorFlag::MuslNoLongerAvailable);
        }

        if app.has_bundled_man_pages() {
            flags.push(DoctorFlag::BundledManPages);
        }
        if app.has_bundled_completions() {
            flags.push(DoctorFlag::BundledCompletions);
        }

        if !flags.is_empty() {
            flagged.push(FlaggedApp {
                id: app.id.clone(),
                release_date: date_str,
                version: version_str,
                flags,
            });
        }
    }

    use std::io::IsTerminal;
    print_table(&flagged, std::io::stdout().is_terminal());
    Ok(())
}

fn load_token(provider: &str, token: Option<String>) -> Result<Option<String>> {
    match &token {
        Some(_) => log::info!("msg={provider}-token-loaded"),
        None => log::warn!("msg={provider} token not found; app may hit API rate limits"),
    }
    Ok(token)
}

enum DoctorFlag {
    PotentiallyUnmaintained,
    MuslNowAvailable,
    MuslNoLongerAvailable,
    BundledManPages,
    BundledCompletions,
}

impl DoctorFlag {
    fn label(&self) -> &str {
        match self {
            Self::PotentiallyUnmaintained => "unmaintained",
            Self::MuslNowAvailable => "musl+",
            Self::MuslNoLongerAvailable => "musl-",
            Self::BundledManPages => "man:bundled",
            Self::BundledCompletions => "comp:bundled",
        }
    }

    fn colored_label(&self, use_color: bool) -> String {
        if !use_color {
            return self.label().to_string();
        }
        let (code, reset) = ("\x1b[", "\x1b[0m");
        let color = match self {
            Self::PotentiallyUnmaintained => "1;33m", // bold yellow
            Self::MuslNowAvailable => "32m",          // green
            Self::MuslNoLongerAvailable => "31m",     // red
            Self::BundledManPages => "36m",           // cyan
            Self::BundledCompletions => "35m",        // magenta
        };
        format!("{}{}{}{}", code, color, self.label(), reset)
    }
}

struct FlaggedApp {
    id:           String,
    release_date: String,
    version:      String,
    flags:        Vec<DoctorFlag>,
}

fn parse_url_parts(url: &str) -> Option<(&str, &str, &str)> {
    let stripped = url
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let mut parts = stripped.splitn(3, '/');
    let host = parts.next()?;
    let owner = parts.next()?;
    let repo = parts.next()?.trim_end_matches('/');
    Some((host, owner, repo))
}

fn fetch_release(
    url: &str, gh_token: Option<String>, cb_token: Option<String>, gl_token: Option<String>,
    offline: bool,
) -> Result<ReleaseMetadata> {
    let (host, owner, repo) =
        parse_url_parts(url).ok_or_else(|| anyhow!("cannot parse url: {}", url))?;
    let client: Box<dyn RelgetClient> = match host {
        "github.com" => Box::new(GithubClient::new(gh_token, offline)),
        "codeberg.org" => Box::new(CodebergClient::new(cb_token, offline)),
        "gitlab.com" => Box::new(GitlabClient::new(gl_token, offline)),
        h => return Err(anyhow!("unknown host: {}", h)),
    };
    client.latest_release(owner, repo)
}

fn release_has_x86_musl(asset_names: &[String]) -> bool {
    // Match musl for x86_64/amd64 only. If no arch marker is present in the name, assume
    // x86_64. Explicitly exclude known non-x86_64 arch strings to avoid false positives.
    asset_names.iter().any(|n| {
        let n = n.to_lowercase();
        n.contains("musl")
            && !n.contains("aarch64")
            && !n.contains("arm64")
            && !n.contains("i686")
            && !n.contains("armv7")
            && !n.contains("arm-")
    })
}

fn release_date(release: &ReleaseMetadata) -> Option<DateTime<Utc>> {
    // GitHub/Codeberg: "published_at"; GitLab: "released_at" then "created_at"
    ["published_at", "released_at", "created_at"]
        .iter()
        .find_map(|key| {
            release.data[*key]
                .as_str()
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|d| d.with_timezone(&Utc))
        })
}

fn print_table(apps: &[FlaggedApp], use_color: bool) {
    if apps.is_empty() {
        println!("All apps look healthy.");
        return;
    }

    let id_w = apps.iter().map(|a| a.id.len()).max().unwrap_or(6).max(6);
    let date_w = 12usize;
    let ver_w = apps
        .iter()
        .map(|a| a.version.len())
        .max()
        .unwrap_or(7)
        .max(7);

    println!(
        "{:<id_w$}  {:<date_w$}  {:<ver_w$}  FLAGS",
        "APP ID",
        "RELEASE DATE",
        "VERSION",
        id_w = id_w,
        date_w = date_w,
        ver_w = ver_w
    );
    println!(
        "{0:-<id_w$}  {0:-<date_w$}  {0:-<ver_w$}  {0:-<30}",
        "",
        id_w = id_w,
        date_w = date_w,
        ver_w = ver_w
    );

    for app in apps {
        let flags_str = app
            .flags
            .iter()
            .map(|f| f.colored_label(use_color))
            .collect::<Vec<_>>()
            .join(", ");
        println!(
            "{:<id_w$}  {:<date_w$}  {:<ver_w$}  {}",
            app.id,
            app.release_date,
            app.version,
            flags_str,
            id_w = id_w,
            date_w = date_w,
            ver_w = ver_w
        );
    }
}
