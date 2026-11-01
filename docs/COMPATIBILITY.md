# Compatibility and status labels

Copyright (c) 2026 c12n Apps — GPL-2.0-or-later

Every firmware in the NextTouch picker carries exactly one status label.
Labels are claims, and claims need evidence. This file is the rulebook
(the vocabulary adapts LightTouch's compatibility ladder — a run climbs
`admitted` → `lit` → `tools` → `ui` → `clean` — to release labels).

| Label | Meaning | Evidence required |
| --- | --- | --- |
| **Tested** | Boots to the home screen and stays responsive ≥ 10 minutes on the current release, then shuts down cleanly (`clean` on the ladder). | CI boot test + manual sign-off on at least one reference host, recorded in this file with the NextTouch version and host platform. |
| **Tested, slow** | As Tested, but noticeably slow on reference hardware. | Same as Tested, plus the slowness note. |
| **Experimental** | The engine can start the boot process; it may panic, hang or never reach the home screen. | At least one real boot attempt on the current engine, with the log kept alongside the compatibility record. |
| **Not booting yet** | Known to fail before the home screen, or the engine has no machine for the device's SoC at all. The picker's Run action is disabled. | A documented reason (here: no A5/A6/A7 machine exists in the engine yet). |

## v0.1.0 state

- Legacy firmwares (iPhone OS 1 – iOS 7) are labelled with the **baseline
  LightTouch engine's** published statuses, and say so: v0.1.0 does not
  integrate the engine, so no legacy label is a NextTouch boot claim.
  iPod touch 2G 3.1.3/4.2.1, iPod touch 3G 4.2.1, iPod touch 4G 6.1.6 are
  Tested upstream; iPhone 3GS 5.1.1 and iPhone 4 4.2.1/4.3.5 are Tested,
  slow; iPod touch 1G and iPhone 2G 3.1.3 and iPad 1 5.1.1 are
  Experimental (upstream: can be prepared and tried; the 1G/2G pair is
  reported to stop at a white screen after setup).
- **All iOS 8 / 9 / 10 firmwares are "Not booting yet"** — 40 entries.
  The engine implements S5L8900–S5L8930 (A4) only. No exceptions, no
  "partially boots" wording until a boot attempt produces a log.

## The internal stage ladder

Behind the labels, each catalog entry carries a `stage` — the
engineering ladder `planned → skeleton → kernel → userland →
springboard → usable → qualified`, bound to the M1–M6 boot milestones
in `docs/ROADMAP.md` (kernel = M1–M2, userland = M3, springboard =
M4–M5, usable = M6 + stability, qualified = the full suite + manual
sign-off). Legacy entries carry the stage the baseline engine has
proven for them (Tested → qualified, Tested slow → usable,
Experimental → skeleton). **Every iOS 8/9/10 entry is `planned` in
v0.1.0.** CI must reject a stage of springboard or higher that has no
recorded milestone test result in the compatibility record.

## Changing a label

A label change requires: the boot evidence above, a catalog regeneration
(`tools/gen_catalog.py`, status curation lives there), a note in the
release notes calling out the move (including regressions to
"Not booting yet"), and this file updated in the same commit.
