// NextTouch library manifest — Copyright (c) 2026 c12n Apps
// SPDX-License-Identifier: GPL-2.0-or-later
//
//! The device library: one entry per prepared emulated device, stored as
//! `library.json` in the app-private data folder.
//!
//! Audit rules implemented here:
//! - Each device gets a RANDOM local id — never the guest UDID/ECID/serial.
//! - The prepared image's SHA-256 is pinned in the manifest; verification
//!   re-hashes the file and compares.
//! - A corrupt manifest is preserved as `library.corrupt-<n>.json` and an
//!   empty library is started — the file is never silently deleted and a
//!   device image is never silently overwritten.
//! - On-disk layout per device mirrors the baseline: immutable `base/`
//!   plus writable `state/` under `devices/<id>/` (engine phase).

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum LibraryError {
    Io(std::io::Error),
    Json(serde_json::Error),
    /// The pinned hash does not match the file on disk.
    HashMismatch { expected: String, actual: String },
    NotFound(String),
}

impl std::fmt::Display for LibraryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LibraryError::Io(e) => write!(f, "library I/O error: {e}"),
            LibraryError::Json(e) => write!(f, "library JSON error: {e}"),
            LibraryError::HashMismatch { expected, actual } => {
                write!(f, "prepared image hash mismatch (expected {expected}, got {actual})")
            }
            LibraryError::NotFound(id) => write!(f, "library device not found: {id}"),
        }
    }
}

impl std::error::Error for LibraryError {}

impl From<std::io::Error> for LibraryError {
    fn from(e: std::io::Error) -> Self {
        LibraryError::Io(e)
    }
}
impl From<serde_json::Error> for LibraryError {
    fn from(e: serde_json::Error) -> Self {
        LibraryError::Json(e)
    }
}

/// One prepared device in the library.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LibraryDevice {
    /// Random local id (32 hex chars). Never a guest hardware identifier.
    pub id: String,
    /// User-facing name, e.g. "My iPhone 4S".
    pub name: String,
    /// Catalog device identifier, e.g. `iPhone4,1`.
    pub device_identifier: String,
    /// Catalog firmware id, e.g. `n94ap-12H321`.
    pub firmware_id: String,
    /// Firmware version string, e.g. `8.4.1`.
    pub firmware_version: String,
    /// SHA-256 pinned at preparation time for the prepared base image.
    /// Empty until the engine phase prepares an image.
    pub prepared_sha256: String,
    /// True after a hard kill / crash: next launch should offer the
    /// snapshot path instead of silently booting a possibly-dirty image.
    pub dirty: bool,
    /// Creation timestamp (UTC ISO-8601) supplied by the caller.
    pub created_at_utc: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Library {
    pub format_version: u32,
    pub devices: Vec<LibraryDevice>,
}

impl Default for Library {
    fn default() -> Self {
        Library { format_version: crate::FORMAT_VERSION, devices: Vec::new() }
    }
}

impl Library {
    pub fn path_in(data_dir: &Path) -> PathBuf {
        data_dir.join("library.json")
    }

    /// Directory holding one device's base/state (engine phase).
    pub fn device_dir(data_dir: &Path, id: &str) -> PathBuf {
        data_dir.join("devices").join(id)
    }

    /// Load the manifest. Missing file -> empty library. Corrupt file ->
    /// preserved as `library.corrupt-<n>.json`, empty library returned.
    pub fn load(data_dir: &Path) -> Library {
        let path = Self::path_in(data_dir);
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(_) => return Library::default(),
        };
        match serde_json::from_str::<Library>(&text) {
            Ok(l) => l,
            Err(_) => {
                crate::settings::preserve_corrupt(&path);
                Library::default()
            }
        }
    }

    pub fn save(&self, data_dir: &Path) -> Result<(), LibraryError> {
        std::fs::create_dir_all(data_dir)?;
        let text = serde_json::to_string_pretty(self)?;
        std::fs::write(Self::path_in(data_dir), text)?;
        Ok(())
    }

    /// Add a device record with a fresh random id. Returns the id.
    pub fn add_device(
        &mut self,
        name: &str,
        device_identifier: &str,
        firmware_id: &str,
        firmware_version: &str,
        created_at_utc: &str,
    ) -> String {
        let id = crate::random_id_hex(&format!("{device_identifier}:{firmware_id}:{created_at_utc}"));
        self.devices.push(LibraryDevice {
            id: id.clone(),
            name: name.to_string(),
            device_identifier: device_identifier.to_string(),
            firmware_id: firmware_id.to_string(),
            firmware_version: firmware_version.to_string(),
            prepared_sha256: String::new(),
            dirty: false,
            created_at_utc: created_at_utc.to_string(),
        });
        id
    }

    pub fn remove_device(&mut self, id: &str) -> Result<(), LibraryError> {
        let before = self.devices.len();
        self.devices.retain(|d| d.id != id);
        if self.devices.len() == before {
            return Err(LibraryError::NotFound(id.to_string()));
        }
        Ok(())
    }

    pub fn device(&self, id: &str) -> Option<&LibraryDevice> {
        self.devices.iter().find(|d| d.id == id)
    }

    /// Verify a prepared image file against the pinned hash.
    pub fn verify_prepared_image(&self, id: &str, image_path: &Path) -> Result<(), LibraryError> {
        let dev = self
            .device(id)
            .ok_or_else(|| LibraryError::NotFound(id.to_string()))?;
        let bytes = std::fs::read(image_path)?;
        let actual = crate::sha256_hex(&bytes);
        if actual != dev.prepared_sha256 {
            return Err(LibraryError::HashMismatch {
                expected: dev.prepared_sha256.clone(),
                actual,
            });
        }
        Ok(())
    }
}
