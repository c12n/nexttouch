// NextTouch catalog tests — Copyright (c) 2026 c12n Apps
// SPDX-License-Identifier: GPL-2.0-or-later
//
// These tests run against the real generated assets/catalog.json. They are
// the release-gate "evals" for the catalog: identity, Apple-host URLs,
// hash discipline and honest status labels.

use nexttouch_catalog::{Catalog, FirmwareStatus, Stage, EMBEDDED_CATALOG_JSON, SCHEMA};

fn catalog() -> Catalog {
    Catalog::from_json(EMBEDDED_CATALOG_JSON).expect("embedded catalog must parse")
}

#[test]
fn embedded_catalog_parses_and_matches_schema() {
    let c = catalog();
    assert_eq!(c.schema, SCHEMA);
    assert!(c.catalog_version >= 1);
    assert!(!c.generated_at_utc.is_empty());
    assert!(c.source.contains("ipsw.me"), "source must record ipsw.me");
    assert!(c.devices.len() >= 20, "expected legacy + 8/9/10 device set");
}

#[test]
fn every_entry_is_well_formed() {
    let c = catalog();
    for d in &c.devices {
        assert!(!d.identifier.is_empty(), "device identifier missing");
        assert!(!d.board.is_empty(), "{}: board missing", d.identifier);
        assert!(!d.soc.is_empty(), "{}: soc missing", d.identifier);
        assert!(!d.firmwares.is_empty(), "{}: no firmwares", d.identifier);
        for f in &d.firmwares {
            // URL discipline: if a URL exists it must be an Apple host.
            if f.download_url.is_some() {
                assert!(
                    f.url_is_apple_host(),
                    "{} {:?}: download URL is not on an Apple host: {:?}",
                    d.identifier,
                    f.version,
                    f.download_url
                );
                assert!(
                    f.ipsw_filename.as_deref().unwrap_or("").ends_with(".ipsw"),
                    "{} {:?}: ipsw filename malformed",
                    d.identifier,
                    f.version
                );
            }
            // Hash discipline: a hash is either a full 64-hex SHA-256 or
            // null WITH a recorded reason. Never a partial/invented hash.
            match &f.sha256 {
                Some(h) => {
                    assert_eq!(h.len(), 64, "{} {:?}: sha256 must be 64 hex chars", d.identifier, f.version);
                    assert!(h.chars().all(|ch| ch.is_ascii_hexdigit()), "{} {:?}: sha256 must be hex", d.identifier, f.version);
                }
                None => {
                    assert!(
                        f.sha256_note.as_deref().unwrap_or("").len() > 3,
                        "{} {:?}: sha256 is null without an explanatory note",
                        d.identifier,
                        f.version
                    );
                }
            }
            // Confirmed entries carry id + build; ids follow <board>-<build>.
            if f.version.is_some() {
                let id = f.id.as_deref().unwrap_or("");
                let build = f.build.as_deref().unwrap_or("");
                assert!(!build.is_empty(), "{} {:?}: build missing", d.identifier, f.version);
                assert_eq!(
                    id,
                    format!("{}-{}", d.board, build),
                    "{} {:?}: id must be <board>-<build>",
                    d.identifier,
                    f.version
                );
            }
        }
    }
}

#[test]
fn ios_8_9_10_entries_never_claim_to_boot() {
    let c = catalog();
    let mut modern = 0;
    for d in &c.devices {
        for f in &d.firmwares {
            if f.ios_major.map(|m| m >= 8).unwrap_or(false) {
                modern += 1;
                assert_eq!(
                    f.status,
                    FirmwareStatus::NotBootingYet,
                    "{} {} must be labelled Not booting yet in v0.1.0",
                    d.identifier,
                    f.version.as_deref().unwrap_or("?")
                );
                assert_eq!(
                    f.stage,
                    Stage::Planned,
                    "{} {} must start at stage 'planned' in v0.1.0",
                    d.identifier,
                    f.version.as_deref().unwrap_or("?")
                );
            }
        }
    }
    assert!(modern >= 30, "expected the full iOS 8/9/10 set, got {modern}");
}

#[test]
fn legacy_statuses_are_from_the_baseline_vocabulary() {
    let c = catalog();
    // iPod touch 2G on 3.1.3 is LightTouch's main tested target.
    let ipod2 = c.device("iPod2,1").expect("iPod2,1 present");
    let fw = ipod2
        .firmwares
        .iter()
        .find(|f| f.version.as_deref() == Some("3.1.3"))
        .expect("3.1.3 present");
    assert_eq!(fw.status, FirmwareStatus::Tested);
    // iPhone 3GS on 5.1.1 is documented "Tested, slow" upstream.
    let gs = c.device("iPhone2,1").expect("iPhone2,1 present");
    let fw = gs
        .firmwares
        .iter()
        .find(|f| f.version.as_deref() == Some("5.1.1"))
        .expect("5.1.1 present");
    assert_eq!(fw.status, FirmwareStatus::TestedSlow);
}

#[test]
fn quick_start_defaults_resolve() {
    let c = catalog();
    let qs = c.quick_start_entries();
    assert_eq!(qs.len(), 3, "the three SPEC quick-start defaults must resolve");
    let names: Vec<&str> = qs.iter().map(|(d, _)| d.identifier.as_str()).collect();
    assert_eq!(names, vec!["iPhone4,1", "iPad2,1", "iPhone5,1"]);
    for (_, f) in &qs {
        assert_eq!(f.version.as_deref(), Some("8.4.1"));
        assert_eq!(f.status, FirmwareStatus::NotBootingYet);
    }
}

#[test]
fn a7_chip_id_is_s5l8960_not_t7000() {
    // Regression guard for the feasibility audit's correction: T7000 is A8.
    let c = catalog();
    let s = c.device("iPhone6,1").expect("iPhone 5s present");
    assert_eq!(s.soc_chip, "S5L8960");
    let six = c.device("iPhone7,2").expect("iPhone 6 present");
    assert_eq!(six.soc_chip, "T7000");
}

#[test]
fn search_finds_devices() {
    let c = catalog();
    assert!(c.search_devices("4s").iter().any(|d| d.identifier == "iPhone4,1"));
    assert!(c.search_devices("n94ap").iter().any(|d| d.identifier == "iPhone4,1"));
    assert_eq!(c.search_devices("").len(), c.devices.len());
}
