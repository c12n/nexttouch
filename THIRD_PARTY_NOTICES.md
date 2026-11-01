# Third-party notices

NextTouch — Copyright (c) 2026 c12n Apps. NextTouch itself is licensed
under the GNU General Public License, version 2 or (at your option) any
later version; see `LICENSE`. This file records the third-party works it
builds on, adapts, or bundles. Licence texts for the GPL family ship in
`LICENSE` (GPL-2.0); the notices below are attribution, not a substitute
for each project's own licence files, which are preserved in their
source repositories.

## Baselines (engine family — engine integration is a roadmap phase)

- **LightTouch** — https://github.com/blake-lucas/LightTouch
  GPL-2.0-or-later. NextTouch's architecture follows LightTouch: a Rust
  workspace (catalog / core / firmware / services) for desktop, a separate
  Kotlin Android app, a shared firmware catalog in the
  `<board>-<build>` id scheme, per-device records with an immutable
  prepared `base/` and writable `state/`, and compatibility labels taken
  from LightTouch's `docs/COMPATIBILITY.md`. The legacy catalog statuses
  in v0.1.0 (Tested / Tested, slow / Experimental) are LightTouch's own
  published statuses for those firmwares.
- **mada-02/iOS-Emulator** — https://github.com/mada-02/iOS-Emulator
  GPL-2.0-or-later. Ideas adopted: CLI parity for scripted runs and the
  one-worker-process-per-device session model. (Its per-device MITM web
  proxy is deliberately NOT adopted — NextTouch installs no root CA into
  any guest.)
- **qemu-ios** — https://github.com/blake-lucas/qemu-ios (fork lineage:
  devos50/qemu-ios → samhenrigold/qemu-ios), GPL-2.0. The emulation
  engine family NextTouch extends; implements S5L8900–S5L8930 (A4).
- **QEMU** — https://www.qemu.org — GPL-2.0.
- **libimobiledevice / libusbmuxd / libplist** — LGPL-2.1-or-later /
  GPL-2.0 components used by the baseline's device services (engine
  phase for NextTouch).

## iOS JIT substrate (compiled into the v0.1.0 IPA)

- **Husk** — https://github.com/Leviidev/Husk — GPL-2.0-or-later.
  The JIT method and the sources under `ios/ios-jit/` are adapted from
  Husk: the breakpoint RPC (`husk-brk.S` → `nexttouch-brk.S`, protocol:
  `brk #0xf00d`, command in x16), the csops/`CS_DEBUGGED` attach
  detection and orchestration shape (`JITBootstrap.swift` →
  `NextTouchJIT.c`/`JITBridge.swift`), and the four entitlements in
  `ios/NextTouch/NextTouch.entitlements` (copied exactly). Husk's
  allocator core (`src/ios-jit/husk-ios-jit.c`) is the designated source
  for the engine-phase port of the region allocator.
  Husk's file headers note that its allocator is derived from
  **AetherPS4**'s JIT allocator (GPL family) and that QEMU's
  `tcg/region.c` split-W^X work comes via QEMU (GPL-2.0).
- **StikDebug / StikJIT** (external JIT enablers the app interoperates
  with; NOT bundled or linked): StikDebug is AGPL-3.0 and remains a
  separate install, exactly as in Husk's design. The debugger-side
  dispatch script lineage (StikDebug's universal script → Husk's
  `husk-jit.js` → NextTouch's `nexttouch-jit.js`, currently a documented
  skeleton) is recorded here and in `ios/ios-jit/nexttouch-jit.js`.

## Libraries bundled in v0.1.0 builds

- **egui / eframe** (desktop) — MIT OR Apache-2.0.
- **serde / serde_json** (Rust core) — MIT OR Apache-2.0.
- **Kotlin, Jetpack Compose, AndroidX** (Android) — Apache-2.0.
- **kotlinx.serialization** (Android) — Apache-2.0.
- **Swift / SwiftUI** (iOS) — Apple platform frameworks, not distributed.

## Data sources

- **ipsw.me** (https://ipsw.me) — firmware metadata API: IPSW download
  URLs and SHA-256 hashes recorded in `assets/catalog.json` at
  generation time, and device/board data cross-checks. Firmware itself
  is Apple's and is never bundled or redistributed by NextTouch.
- **The Apple Wiki** (theiphonewiki successor) — firmware-key
  availability research behind the roadmap's decryption assessment
  (no key material is reproduced in this project).

## Trademarks

iPhone, iPad, iPod touch and iOS are trademarks of Apple Inc. / Cisco as
applicable. This project is not affiliated with or endorsed by Apple.
