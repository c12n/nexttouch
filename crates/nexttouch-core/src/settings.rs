// NextTouch settings — Copyright (c) 2026 c12n Apps
// SPDX-License-Identifier: GPL-2.0-or-later
//
//! User settings, persisted as JSON inside the app-private data folder
//! (never the Windows registry, never UserDefaults on desktop).
//! The theme is a three-state preference — Follow system is the default
//! and is persisted as a preference, not as a resolved theme.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Three-state theme preference (DeepSeek audit §themes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ThemePreference {
    /// Follow the OS theme; the app fallback when the OS reports none is dark.
    #[default]
    System,
    Light,
    Dark,
}

impl ThemePreference {
    pub fn label(&self) -> &'static str {
        match self {
            ThemePreference::System => "Follow system",
            ThemePreference::Light => "Light",
            ThemePreference::Dark => "Dark",
        }
    }
    pub fn all() -> [ThemePreference; 3] {
        [ThemePreference::System, ThemePreference::Light, ThemePreference::Dark]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub format_version: u32,
    pub theme: ThemePreference,
    /// Guest networking is OFF by default (audit rule); the engine phase
    /// reads this flag, the v0.1.0 UI exposes it read-only in Settings.
    pub guest_network_enabled: bool,
    /// Diagnostic logging is opt-in and local-only.
    pub diagnostic_logging: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            format_version: crate::FORMAT_VERSION,
            theme: ThemePreference::System,
            guest_network_enabled: false,
            diagnostic_logging: false,
        }
    }
}

impl Settings {
    pub fn path_in(data_dir: &Path) -> PathBuf {
        data_dir.join("settings.json")
    }

    /// Load settings from the data dir. A missing file yields defaults.
    /// A corrupt file is preserved as `settings.corrupt-<n>.json` (never
    /// silently deleted) and defaults are returned.
    pub fn load(data_dir: &Path) -> Settings {
        let path = Self::path_in(data_dir);
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(_) => return Settings::default(),
        };
        match serde_json::from_str::<Settings>(&text) {
            Ok(s) => s,
            Err(_) => {
                preserve_corrupt(&path);
                Settings::default()
            }
        }
    }

    pub fn save(&self, data_dir: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(data_dir)?;
        let text = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(Self::path_in(data_dir), text)
    }
}

/// Rename a corrupt JSON file aside (`<name>.corrupt-<n>.json`), trying
/// increasing suffixes so an earlier preserved copy is never overwritten.
pub(crate) fn preserve_corrupt(path: &Path) -> Option<PathBuf> {
    for n in 0..1000 {
        let candidate = path.with_file_name(format!(
            "{}.corrupt-{}.json",
            path.file_name().unwrap_or_default().to_string_lossy().trim_end_matches(".json"),
            n
        ));
        if !candidate.exists() {
            return std::fs::rename(path, &candidate).ok().map(|_| candidate);
        }
    }
    None
}
