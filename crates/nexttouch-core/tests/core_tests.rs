// NextTouch core tests — Copyright (c) 2026 c12n Apps
// SPDX-License-Identifier: GPL-2.0-or-later

use nexttouch_core::library::Library;
use nexttouch_core::privacy::{delete_all_data, download_url_allowed, log_safe_url, redact_for_log};
use nexttouch_core::settings::{Settings, ThemePreference};
use nexttouch_core::sha256_hex;

fn temp_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("nexttouch-core-test-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn sha256_matches_nist_vector() {
    // FIPS 180-4 example: SHA-256("abc")
    assert_eq!(
        sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        sha256_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn settings_default_to_follow_system_and_roundtrip() {
    let dir = temp_dir("settings");
    let s = Settings::load(&dir);
    assert_eq!(s.theme, ThemePreference::System);
    assert!(!s.guest_network_enabled, "guest networking must default off");
    assert!(!s.diagnostic_logging);
    let mut s2 = s.clone();
    s2.theme = ThemePreference::Dark;
    s2.save(&dir).unwrap();
    let loaded = Settings::load(&dir);
    assert_eq!(loaded.theme, ThemePreference::Dark);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn corrupt_settings_are_preserved_not_deleted() {
    let dir = temp_dir("corrupt");
    std::fs::write(Settings::path_in(&dir), "{ not json").unwrap();
    let s = Settings::load(&dir);
    assert_eq!(s.theme, ThemePreference::System);
    let names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    assert!(
        names.iter().any(|n| n.contains("corrupt")),
        "corrupt settings file must be preserved, found: {names:?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn library_random_ids_pinned_hash_and_corrupt_preservation() {
    let dir = temp_dir("library");
    let mut lib = Library::load(&dir);
    let id = lib.add_device("My 4S", "iPhone4,1", "n94ap-12H321", "8.4.1", "2026-11-01T00:00:00Z");
    assert_eq!(id.len(), 32);
    assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    // The id must never be/contain a hardware-style identifier.
    assert_ne!(id, "iPhone4,1");
    lib.devices[0].prepared_sha256 = sha256_hex(b"fake-image");
    lib.save(&dir).unwrap();

    let image = dir.join("img.bin");
    std::fs::write(&image, b"fake-image").unwrap();
    let lib2 = Library::load(&dir);
    assert!(lib2.verify_prepared_image(&id, &image).is_ok());
    std::fs::write(&image, b"tampered").unwrap();
    assert!(lib2.verify_prepared_image(&id, &image).is_err());

    // Corrupt manifest: preserved aside, empty library returned.
    std::fs::write(Library::path_in(&dir), "garbage").unwrap();
    let lib3 = Library::load(&dir);
    assert!(lib3.devices.is_empty());
    let names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    assert!(names.iter().any(|n| n.contains("library.corrupt")), "found: {names:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn privacy_rules_hold() {
    assert!(download_url_allowed("https://updates.cdn-apple.com/x/y.ipsw"));
    assert!(download_url_allowed("https://secure-appldnld.apple.com/y.ipsw"));
    assert!(!download_url_allowed("https://example.com/y.ipsw"));
    assert!(!download_url_allowed("https://apple.com.evil.example/y.ipsw"));

    let safe = log_safe_url("https://updates.cdn-apple.com/2019/ios/abc/file.ipsw?token=secret");
    assert!(safe.starts_with("https://updates.cdn-apple.com"));
    assert!(!safe.contains("secret"), "query strings must never be logged");

    let redacted = redact_for_log("downloading https://updates.cdn-apple.com/a/b.ipsw?t=1 now");
    assert!(!redacted.contains("b.ipsw"));
    let redacted_id = redact_for_log("device 00008030001a2b3c4d5e6f77889900112233445566 seen");
    assert!(redacted_id.contains("<id>"), "long hex ids must be redacted: {redacted_id}");
}

#[test]
fn delete_all_data_removes_tree() {
    let dir = temp_dir("wipe");
    std::fs::write(dir.join("settings.json"), "{}").unwrap();
    delete_all_data(&dir).unwrap();
    assert!(!dir.exists());
}
