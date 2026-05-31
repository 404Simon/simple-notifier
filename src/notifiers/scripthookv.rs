use regex::Regex;
use ureq;

use crate::notifier::{Notification, Notifier};
use crate::storage::Storage;

const URL: &str = "http://www.dev-c.com/gtav/scripthookv/";
const STORAGE_KEY: &str = "scripthookv_version";

pub struct ScriptHookV;

impl Notifier for ScriptHookV {
    fn name(&self) -> &str {
        "scripthookv"
    }

    fn check(&self, storage: &mut Storage) -> Option<Notification> {
        let body = match fetch_page() {
            Ok(b) => b,
            Err(e) => {
                eprintln!("[scripthookv] fetch error: {e}");
                return None;
            }
        };

        let version = match extract_version(&body) {
            Some(v) => v,
            None => {
                eprintln!("[scripthookv] could not extract version from page");
                return None;
            }
        };

        let prev = storage.get(STORAGE_KEY).map(|s| s.to_string());
        if prev.as_deref() == Some(&version) {
            return None;
        }

        storage.set(STORAGE_KEY, &version);

        let (title, body_text) = match &prev {
            Some(old) => (
                format!("ScriptHookV updated to {version}"),
                format!(
                    "A new version of ScriptHookV is available: {version}\n\nPrevious version: {old}\n\n{URL}"
                ),
            ),
            None => (
                format!("ScriptHookV {version} detected"),
                format!(
                    "Initial ScriptHookV version detected: {version}\n\n{URL}"
                ),
            ),
        };

        Some(Notification {
            title,
            body: body_text,
        })
    }
}

fn fetch_page() -> Result<String, String> {
    let resp = ureq::get(URL)
        .header("User-Agent", "simple-notifier/0.1")
        .call()
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    resp.into_body()
        .read_to_string()
        .map_err(|e| format!("failed to read response body: {e}"))
}

fn extract_version(html: &str) -> Option<String> {
    let re = Regex::new(r"v?(\d+\.\d+\.\d+(?:\.\d+)?)").ok()?;

    let downloads_section = html.find("downloadsSection")
        .or_else(|| html.find("download"))
        .unwrap_or(0);

    let search_start = downloads_section.saturating_sub(500);
    let search_window = &html[search_start..];

    let mut candidates: Vec<&str> = re
        .captures_iter(search_window)
        .map(|cap| cap.get(1).map(|m| m.as_str()).unwrap_or(""))
        .filter(|v| v.matches('.').count() >= 2)
        .collect();

    if candidates.is_empty() {
        candidates = re
            .captures_iter(html)
            .map(|cap| cap.get(1).map(|m| m.as_str()).unwrap_or(""))
            .filter(|v| v.matches('.').count() >= 2)
            .collect();
    }

    candidates.sort_by_key(|v| -(v.len() as i32));
    candidates.into_iter().next().map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_version() {
        let html = r#"<a href="/gtav/scripthookv/ScriptHookV_1.0.335.2.zip">Download</a>"#;
        assert_eq!(extract_version(html), Some("1.0.335.2".to_string()));
    }

    #[test]
    fn test_extract_version_in_text() {
        let html = r#"Script Hook V v1.0.335.2 is available"#;
        assert_eq!(extract_version(html), Some("1.0.335.2".to_string()));
    }

    #[test]
    fn test_extract_version_near_download() {
        let html = format!(
            "{}{}{}",
            "x".repeat(600),
            r#"<div class="downloadsSection"><a href="ScriptHookV_1.0.340.0.zip">"#,
            "y".repeat(200)
        );
        assert_eq!(extract_version(&html), Some("1.0.340.0".to_string()));
    }
}
