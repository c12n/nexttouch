#!/usr/bin/env python3
# Copyright (c) 2026 c12n Apps
# SPDX-License-Identifier: GPL-2.0-or-later
"""Generate crates/nexttouch-catalog/assets/catalog.json from the ipsw.me API.

Every firmware entry (legacy iPhone OS 1 - iOS 7 and the iOS 8/9/10 targets)
is pulled from https://api.ipsw.me/v4/device/<identifier>?type=ipsw so the
download URL and SHA-256 come from ipsw.me (which mirrors Apple's own
metadata) at generation time -- nothing is invented. If a firmware a device
is curated for is not listed by ipsw.me, the entry is still emitted with
"sha256": null and a "sha256_note" explaining why, so the picker can show
the entry without ever fabricating a hash or URL (then download_url is null
too and the UI disables the download button).

Status labels are curated here, never derived from ipsw.me:
- Legacy firmwares carry the status LightTouch (the baseline engine) reports
  for them: "tested", "tested_slow" or "experimental" (can be prepared and
  tried, not yet checked).
- Every iOS 8/9/10 firmware is "not_booting_yet" in v0.1.0: the qemu-ios
  engine implements no A5/A6/A7 SoC yet. The picker must never claim one
  boots. Their "tier" field is a development priority (target = A5,
  experimental = A6, future = arm64), not a boot claim.

Usage: python3 tools/gen_catalog.py [--out PATH] [--offline-fallback]
With --offline-fallback (or if the API is unreachable), the same curated
device set is emitted with url/sha256 null and unverified flags.
"""

import argparse
import json
import sys
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

API = "https://api.ipsw.me/v4/device/{identifier}?type=ipsw"

CATALOG_VERSION = 1
SCHEMA = "nexttouch-catalog/1"

STATUS_TESTED = "tested"
STATUS_TESTED_SLOW = "tested_slow"
STATUS_EXPERIMENTAL = "experimental"
STATUS_NOT_BOOTING = "not_booting_yet"

TIER_LEGACY = "legacy"
TIER_TARGET = "target"
TIER_EXPERIMENTAL = "experimental"
TIER_FUTURE = "future"

# ---------------------------------------------------------------------------
# Curated device set.
# fields: identifier, name, board, soc, soc_chip, arch, display_px, ram_mb,
#         tier, firmwares: [(version, build, status)], quick_start: bool
# (build is the curated Apple build id, used as a fallback when ipsw.me
# does not list that firmware at all -- e.g. it indexes no iPhone OS 3.x
# IPSW for iPod1,1/iPod2,1; the entry then keeps version+build, gets
# id=<board>-<build> per the LightTouch catalog, and records url/sha256
# as null with an explanatory note. Hashes and URLs are never invented.)
# Legacy devices are the exact machines the baseline qemu-ios engine
# implements (ENGINE-FEASIBILITY audit section 1.2), with LightTouch's own
# tested/tested-slow/experimental status for the listed firmwares.
# ---------------------------------------------------------------------------

DEVICES = [
    # ---------------- Legacy (iPhone OS 1 - iOS 7), engine-supported --------
    dict(identifier="iPod1,1", name="iPod touch 1G", board="n45ap",
         soc="S5L8900", soc_chip="S5L8900", arch="armv6",
         display_px=[320, 480], ram_mb=128, tier=TIER_LEGACY,
         firmwares=[("3.1.3", "7E18", STATUS_EXPERIMENTAL)]),
    dict(identifier="iPhone1,1", name="iPhone (2G)", board="m68ap",
         soc="S5L8900", soc_chip="S5L8900", arch="armv6",
         display_px=[320, 480], ram_mb=128, tier=TIER_LEGACY,
         firmwares=[("3.1.3", "7E18", STATUS_EXPERIMENTAL)]),
    dict(identifier="iPod2,1", name="iPod touch 2G", board="n72ap",
         soc="S5L8720", soc_chip="S5L8720", arch="armv6",
         display_px=[320, 480], ram_mb=128, tier=TIER_LEGACY,
         firmwares=[("3.1.3", "7E18", STATUS_TESTED), ("4.2.1", "8C148", STATUS_TESTED)]),
    dict(identifier="iPod3,1", name="iPod touch 3G", board="n18ap",
         soc="S5L8922", soc_chip="S5L8922", arch="armv7",
         display_px=[320, 480], ram_mb=256, tier=TIER_LEGACY,
         firmwares=[("4.2.1", "8C148", STATUS_TESTED)]),
    dict(identifier="iPhone2,1", name="iPhone 3GS", board="n88ap",
         soc="S5L8920", soc_chip="S5L8920", arch="armv7",
         display_px=[320, 480], ram_mb=256, tier=TIER_LEGACY,
         firmwares=[("5.1.1", "9B206", STATUS_TESTED_SLOW)]),
    dict(identifier="iPod4,1", name="iPod touch 4G", board="n81ap",
         soc="A4", soc_chip="S5L8930", arch="armv7",
         display_px=[640, 960], ram_mb=256, tier=TIER_LEGACY,
         firmwares=[("6.1.6", "10B500", STATUS_TESTED)]),
    dict(identifier="iPhone3,1", name="iPhone 4", board="n90ap",
         soc="A4", soc_chip="S5L8930", arch="armv7",
         display_px=[640, 960], ram_mb=512, tier=TIER_LEGACY,
         firmwares=[("4.2.1", "8C148", STATUS_TESTED_SLOW), ("4.3.5", "8L1", STATUS_TESTED_SLOW)]),
    dict(identifier="iPad1,1", name="iPad 1", board="k48ap",
         soc="A4", soc_chip="S5L8930", arch="armv7",
         display_px=[768, 1024], ram_mb=256, tier=TIER_LEGACY,
         firmwares=[("5.1.1", "9B206", STATUS_EXPERIMENTAL)]),
    # ---------------- iOS 8 / 9 / 10 targets (NOT booting yet) --------------
    # A5 family -- first development target (iOS 8.4.1 on iPhone 4S, then 9).
    dict(identifier="iPhone4,1", name="iPhone 4S", board="n94ap",
         soc="A5", soc_chip="S5L8940", arch="armv7",
         display_px=[640, 960], ram_mb=512, tier=TIER_TARGET,
         majors=[8, 9], quick_start=True),
    dict(identifier="iPad2,1", name="iPad 2 (Wi-Fi)", board="k93ap",
         soc="A5", soc_chip="S5L8940", arch="armv7",
         display_px=[768, 1024], ram_mb=512, tier=TIER_TARGET,
         majors=[8, 9], quick_start=True),
    dict(identifier="iPod5,1", name="iPod touch 5", board="n78ap",
         soc="A5", soc_chip="S5L8942", arch="armv7",
         display_px=[640, 1136], ram_mb=512, tier=TIER_TARGET,
         majors=[8, 9]),
    dict(identifier="iPad2,5", name="iPad mini 1 (Wi-Fi)", board="p105ap",
         soc="A5", soc_chip="S5L8942", arch="armv7",
         display_px=[768, 1024], ram_mb=512, tier=TIER_TARGET,
         majors=[8, 9]),
    dict(identifier="iPad3,1", name="iPad 3 (Wi-Fi)", board="j1ap",
         soc="A5X", soc_chip="S5L8945", arch="armv7",
         display_px=[1536, 2048], ram_mb=1024, tier=TIER_TARGET,
         majors=[8, 9]),
    # A6 family -- second target (the only 32-bit path to iOS 10).
    dict(identifier="iPhone5,1", name="iPhone 5 (GSM)", board="n41ap",
         soc="A6", soc_chip="S5L8950", arch="armv7s",
         display_px=[640, 1136], ram_mb=1024, tier=TIER_EXPERIMENTAL,
         majors=[8, 9, 10], quick_start=True),
    dict(identifier="iPhone5,3", name="iPhone 5c (GSM)", board="n48ap",
         soc="A6", soc_chip="S5L8950", arch="armv7s",
         display_px=[640, 1136], ram_mb=1024, tier=TIER_EXPERIMENTAL,
         majors=[8, 9, 10]),
    dict(identifier="iPad3,4", name="iPad 4 (Wi-Fi)", board="p101ap",
         soc="A6X", soc_chip="S5L8955", arch="armv7s",
         display_px=[1536, 2048], ram_mb=1024, tier=TIER_EXPERIMENTAL,
         majors=[8, 9, 10]),
    # arm64 family -- future work (A7+ also needs a Secure Enclave model).
    dict(identifier="iPhone6,1", name="iPhone 5s (GSM)", board="n51ap",
         soc="A7", soc_chip="S5L8960", arch="arm64",
         display_px=[640, 1136], ram_mb=1024, tier=TIER_FUTURE,
         majors=[8, 9, 10]),
    dict(identifier="iPad4,1", name="iPad Air (Wi-Fi)", board="j71ap",
         soc="A7", soc_chip="S5L8965", arch="arm64",
         display_px=[1536, 2048], ram_mb=1024, tier=TIER_FUTURE,
         majors=[8, 9, 10]),
    dict(identifier="iPad4,4", name="iPad mini 2 (Wi-Fi)", board="j85ap",
         soc="A7", soc_chip="S5L8960", arch="arm64",
         display_px=[1536, 2048], ram_mb=1024, tier=TIER_FUTURE,
         majors=[8, 9, 10]),
    dict(identifier="iPhone7,2", name="iPhone 6", board="n61ap",
         soc="A8", soc_chip="T7000", arch="arm64",
         display_px=[750, 1334], ram_mb=1024, tier=TIER_FUTURE,
         majors=[8, 9, 10]),
    dict(identifier="iPad5,3", name="iPad Air 2 (Wi-Fi)", board="j81ap",
         soc="A8X", soc_chip="T7001", arch="arm64",
         display_px=[1536, 2048], ram_mb=2048, tier=TIER_FUTURE,
         majors=[8, 9, 10]),
    dict(identifier="iPod7,1", name="iPod touch 6", board="n102ap",
         soc="A8", soc_chip="T7000", arch="arm64",
         display_px=[640, 1136], ram_mb=1024, tier=TIER_FUTURE,
         majors=[8, 9, 10]),
    dict(identifier="iPhone8,1", name="iPhone 6s", board="n71ap",
         soc="A9", soc_chip="S8000", arch="arm64",
         display_px=[750, 1334], ram_mb=2048, tier=TIER_FUTURE,
         majors=[9, 10]),
    dict(identifier="iPhone9,1", name="iPhone 7", board="d10ap",
         soc="A10 Fusion", soc_chip="T8010", arch="arm64",
         display_px=[750, 1334], ram_mb=2048, tier=TIER_FUTURE,
         majors=[10]),
]

QUICK_START_IDS = ["iPhone4,1:8.4.1", "iPad2,1:8.4.1", "iPhone5,1:8.4.1"]


def version_key(v):
    parts = []
    for chunk in str(v).split("."):
        digits = ""
        for ch in chunk:
            if ch.isdigit():
                digits += ch
            else:
                break
        parts.append(int(digits) if digits else 0)
    return parts


def fetch_firmwares(identifier):
    url = API.format(identifier=urllib.request.quote(identifier, safe=","))
    req = urllib.request.Request(url, headers={"User-Agent": "NextTouch-catalog-gen"})
    with urllib.request.urlopen(req, timeout=30) as resp:
        data = json.loads(resp.read().decode("utf-8"))
    return data.get("firmwares", [])


def os_name_for(version):
    return "iPhone OS" if version_key(version)[0] < 4 else "iOS"


def pick_final_per_major(firmwares, majors):
    """Return {major: firmware dict} with the highest version in each major."""
    best = {}
    for fw in firmwares:
        v = fw.get("version", "")
        if not v:
            continue
        major = version_key(v)[0]
        if major not in majors:
            continue
        if major not in best or version_key(v) > version_key(best[major]["version"]):
            best[major] = fw
    return best


def stage_for(status, ios_major):
    """Internal engineering stage ladder (Claude blueprint):
    planned -> skeleton -> kernel -> userland -> springboard -> usable ->
    qualified. iOS 8/9/10 entries all start at 'planned' in v0.1.0; the
    legacy entries carry the stage the BASELINE engine has proven for
    them. CI must reject any stage >= springboard without a recorded
    boot-test result (M1-M6 milestones, see docs/ROADMAP.md)."""
    if ios_major is not None and ios_major >= 8:
        return "planned"
    return {
        STATUS_TESTED: "qualified",
        STATUS_TESTED_SLOW: "usable",
        STATUS_EXPERIMENTAL: "skeleton",
    }[status]


def make_entry(device, fw, status, verified, curated_version=None, curated_build=None):
    version = fw["version"] if fw else curated_version
    build = fw["buildid"] if fw else curated_build
    entry = {
        "id": "{}-{}".format(device["board"], build) if build else None,
        "version": version,
        "build": build,
        "os_name": os_name_for(version) if version else None,
        "ios_major": version_key(version)[0] if version else None,
        "ipsw_filename": None,
        "download_url": None,
        "sha256": None,
        "sha256_note": None,
        "signed": None,
        "status": status,
        "stage": stage_for(status, version_key(version)[0] if version else None),
    }
    if fw is None:
        entry["sha256_note"] = (
            "ipsw.me does not list an IPSW for this firmware on this device, "
            "so no download URL or SHA-256 is recorded (version/build are "
            "curated from the baseline LightTouch catalog). Nothing is "
            "fabricated -- the engine phase sources this firmware separately."
        )
        return entry
    url = fw.get("url") or None
    entry["download_url"] = url
    if url:
        entry["ipsw_filename"] = url.rsplit("/", 1)[-1]
    entry["signed"] = fw.get("signed")
    sha = fw.get("sha256sum") or None
    if sha:
        entry["sha256"] = sha
    elif not verified:
        entry["sha256_note"] = "unverified: generated without live ipsw.me data"
    else:
        entry["sha256_note"] = "ipsw.me lists no SHA-256 for this firmware"
    return entry


def build_catalog(offline=False):
    devices_out = []
    warnings = []
    any_live = False
    for dev in DEVICES:
        firmwares = []
        if not offline:
            try:
                firmwares = fetch_firmwares(dev["identifier"])
                any_live = True
            except Exception as exc:  # noqa: BLE001 - record and fall back
                warnings.append("{}: ipsw.me fetch failed ({})".format(dev["identifier"], exc))
                firmwares = []
        by_version = {f.get("version"): f for f in firmwares}
        entries = []
        if "firmwares" in dev:  # legacy: explicit versions + statuses
            for version, build, status in dev["firmwares"]:
                fw = by_version.get(version)
                if fw is None and firmwares:
                    warnings.append("{}: version {} not listed by ipsw.me".format(dev["identifier"], version))
                entries.append(make_entry(dev, fw, status, verified=bool(firmwares),
                                          curated_version=version, curated_build=build))
        else:  # iOS 8/9/10: final build of each requested major
            finals = pick_final_per_major(firmwares, dev["majors"])
            for major in dev["majors"]:
                fw = finals.get(major)
                if fw is None:
                    if firmwares:
                        warnings.append("{}: no iOS {} firmware listed by ipsw.me".format(dev["identifier"], major))
                    # Still emit an honest placeholder entry for the major.
                    placeholder = None
                    entries.append({
                        "id": None, "version": None, "build": None,
                        "os_name": "iOS", "ios_major": major,
                        "ipsw_filename": None, "download_url": None,
                        "sha256": None,
                        "sha256_note": "No final iOS {} firmware for this device was returned by the source at generation time.".format(major),
                        "signed": None, "status": STATUS_NOT_BOOTING,
                        "stage": "planned",
                    })
                    _ = placeholder
                else:
                    entries.append(make_entry(dev, fw, STATUS_NOT_BOOTING, verified=True))
        devices_out.append({
            "identifier": dev["identifier"],
            "name": dev["name"],
            "board": dev["board"],
            "soc": dev["soc"],
            "soc_chip": dev["soc_chip"],
            "arch": dev["arch"],
            "display_px": dev["display_px"],
            "ram_mb": dev["ram_mb"],
            "tier": dev["tier"],
            "quick_start": bool(dev.get("quick_start", False)),
            "firmwares": entries,
        })
    verified = any_live and not warnings
    catalog = {
        "schema": SCHEMA,
        "catalog_version": CATALOG_VERSION,
        "generated_at_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "generated_by": "tools/gen_catalog.py",
        "source": "https://api.ipsw.me/v4 (firmware URLs + SHA-256); device/SoC/status curation in tools/gen_catalog.py from LightTouch compatibility docs and the NextTouch engine-feasibility audit",
        "verified": verified,
        "quick_start": QUICK_START_IDS,
        "devices": devices_out,
    }
    return catalog, warnings


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=str(Path(__file__).resolve().parent.parent / "crates/nexttouch-catalog/assets/catalog.json"))
    ap.add_argument("--offline-fallback", action="store_true")
    args = ap.parse_args()
    catalog, warnings = build_catalog(offline=args.offline_fallback)
    for w in warnings:
        print("WARNING:", w, file=sys.stderr)
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(catalog, indent=2) + "\n", encoding="utf-8")
    total_fw = sum(len(d["firmwares"]) for d in catalog["devices"])
    print("wrote {} ({} devices, {} firmwares, verified={})".format(out, len(catalog["devices"]), total_fw, catalog["verified"]))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
