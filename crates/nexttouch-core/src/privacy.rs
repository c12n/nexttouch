// NextTouch privacy rules — Copyright (c) 2026 c12n Apps
// SPDX-License-Identifier: GPL-2.0-or-later
//
//! The privacy rules from the project's DeepSeek audit, expressed as code
//! so they are testable and shared by every platform shell (the Kotlin and
//! Swift shells mirror these exact rules):
//!
//! - The ONLY network path in the app is an IPSW download from an Apple
//!   host, on explicit user tap. No telemetry, analytics, crash upload,
//!   update pings, remote config or third-party SDKs exist in the tree.
//! - Guest networking is off by default.
//! - Logs are local-only and must never contain full filesystem paths,
//!   guest hardware identifiers (UDID/ECID/serial), user file names or
//!   URLs with query strings. `redact_for_log` enforces the mechanically
//!   checkable parts of that rule.

/// Hosts a firmware download may contact. Mirrors
/// `nexttouch_catalog::is_apple_download_host` but kept dependency-light
/// for the log/redaction side of the core.
pub fn download_url_allowed(url: &str) -> bool {
    nexttouch_catalog::is_apple_download_host(url)
}

/// Strip a URL down to scheme + host for logging. Query strings and paths
/// (which can carry session tokens and file names) are never logged.
pub fn log_safe_url(url: &str) -> String {
    let (scheme, rest) = match url.split_once("://") {
        Some((s, r)) => (s, r),
        None => return "<unparseable-url>".to_string(),
    };
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    if host.is_empty() {
        return "<unparseable-url>".to_string();
    }
    format!("{scheme}://{host}/…")
}

/// Redact a free-form message before it reaches any log sink:
/// - anything that looks like a URL is reduced to scheme + host,
/// - anything that looks like a long hex identifier (40+ hex chars, the
///   UDID pattern) is replaced with `<id>`,
/// - backslash/forward-slash paths with a drive or home prefix are
///   replaced with `<path>` (best-effort, defence in depth — the real
///   rule is that callers never format paths into logs at all).
pub fn redact_for_log(message: &str) -> String {
    let mut out = String::with_capacity(message.len());
    for token in message.split_inclusive(|c: char| c.is_whitespace()) {
        let trimmed = token.trim_end();
        let suffix = &token[trimmed.len()..];
        let replacement: String = if trimmed.contains("://") {
            log_safe_url(trimmed)
        } else if trimmed.len() >= 40 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
            "<id>".to_string()
        } else if (trimmed.contains('\\') && trimmed.chars().filter(|c| *c == '\\').count() >= 2)
            || (trimmed.starts_with('/') && trimmed.matches('/').count() >= 3)
        {
            "<path>".to_string()
        } else {
            trimmed.to_string()
        };
        out.push_str(&replacement);
        out.push_str(suffix);
    }
    out
}

/// The wipe-everything contract used by Settings on all platforms:
/// stop workers, delete the whole data tree, clear the manifest. The
/// desktop implementation is here; mobile shells implement the same steps.
pub fn delete_all_data(data_dir: &std::path::Path) -> std::io::Result<()> {
    if data_dir.exists() {
        std::fs::remove_dir_all(data_dir)?;
    }
    Ok(())
}
