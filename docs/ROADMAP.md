# Roadmap

Copyright (c) 2026 c12n Apps — GPL-2.0-or-later

NextTouch's goal is running iOS 8, 9 and 10. The honest starting point
(agreed by the project's engine-feasibility audit, its Claude
architecture pass, and its Gemini/DeepSeek passes): **no public QEMU
machine exists for any SoC that can run iOS 8+.** The newest SoC in the
baseline engine is the A4 (S5L8930). Each step below is therefore a real
engine project, not a configuration change — and picker labels and
internal stages only move when boots are proven against the milestones
below (see also `docs/COMPATIBILITY.md`).

## The version ladder

| Version | Milestone |
| --- | --- |
| **0.1.0** | Foundation: shared catalog + picker + library + themes + privacy on Windows/Android/iOS; iOS JIT substrate (Husk arrangement) compiled into the IPA with an inert prewarm; n94ap machine work not started. |
| **0.2** | **Kernel alive** — the A5 machine skeleton boots the kernel to the M1–M2 milestones on the firmware ladder below. |
| **0.3** | **Userland** — root filesystem mounts, launchd up, shell over serial (M3). |
| **0.4** | **First iOS 8/9 SpringBoard** (M4–M5) on iPhone 4S. |
| **0.5** | **Usable A5** — input works end to end (M6), iOS 8.4.1 and 9.3.5 both usable-class on iPhone 4S; same-silicon siblings (iPad 2, iPod touch 5, iPad mini 1) follow. |
| **0.6** | **First iOS 10 on A6** — the second new SoC (iPhone 5), kernel → userland on 10.3.4. |
| **1.0** | **Qualified matrix** — the supported device/firmware matrix passes the full milestone suite in CI plus manual sign-off. Only the project owner promotes to 1.0. |

A7 and newer (64-bit) are **out of scope** for this ladder: the Secure
Enclave model, IMG4 personalisation and the Metal-era UI make them a
different class of project (see "Later, deliberately" below).

## The firmware ladder (one board, on purpose)

All early work happens on **one board — iPhone 4S (n94ap, A5/S5L8940)** —
climbing firmware versions so machine bugs separate cleanly from OS
bugs:

**5.1.1 → 6.1.3 → 7.1.2 → 8.4.1 → 9.3.5**

- 5.1.1/6.1.3/7.1.2 never shipped for the 4S's A5 in the baseline engine —
  they are *new-machine* bring-up steps chosen because each is closer to
  what the baseline already understands than jumping straight to iOS 8.
- **8.4.1 (12H321)** is the first iOS 8 target.
- **9.3.5 (13G36)** is the A5 headline target: the same machine covers
  iOS 9 with firmware-side work only (kernelpatcher offsets, guest
  tools, rootfs recipes per build).
- iPad 2 Wi-Fi (k93ap) is the modem-free debug twin on the same silicon.

**Step 0, before any device model:** decrypt the n94ap 9.3.5 device tree
and kernelcache and dump every node's `compatible`/`reg`/`interrupts`,
mapping drivers to nodes. That dump is the source of truth for the
machine definition (keys for these final builds are published;
decryption is not a blocker on the A5/A6 iOS 8/9 path).

## Boot milestones — the project's test definitions

Every stage claim is tested against these milestones, in order:

| ID | Milestone |
| --- | --- |
| **M1** | Kernel banner on the serial console. |
| **M2** | Root device found. |
| **M3** | launchd up; a shell answers over serial. |
| **M4** | backboardd presenting (first real frames through the GL bridge). |
| **M5** | SpringBoard in front, idle-stable. |
| **M6** | An injected tap opens Settings (input path proven end to end). |

The catalog carries this ladder internally as each firmware's `stage`:
`planned → skeleton → kernel → userland → springboard → usable →
qualified`, mapped to M1 (kernel), M3 (userland), M5 (springboard),
M6 + stability (usable) and the full suite + sign-off (qualified). The
user-facing picker keeps the three honest labels (Tested / Experimental
/ Not booting yet). **CI rejects any stage ≥ springboard without a
recorded milestone test result** attached to the compatibility record.
In v0.1.0 every iOS 8/9/10 entry is `stage: planned`.

## A5 component plan (0.2–0.5)

- **Implement:** AIC interrupt controller (replaces the A4's VIC),
  timers, UART (verbose serial console from day one), NAND + storage,
  AES/SHA engines. Dual-core Cortex-A9 SMP bring-up. Direct kernel boot
  (no boot-ROM chain); device-tree edits strip GPU/modem/camera/second
  CPU where a stub is not yet honest.
- **Second critical path: storage + data protection.** A fake-UID keybag
  must behave or nothing past launchd mounts.
- **Stub:** clocks/PMIC, sensors, audio.
- **Omit:** modem, Wi-Fi, camera (the iPad 2 twin needs no modem story
  at all; the 4S gets the baseline's fake-modem *pattern*, not its code).
- **Graphics — extend the GL bridge, do not emulate a GPU.** The engine
  family renders through a host OpenGL front end, not an emulated
  PowerVR. iOS 8–10 work is front-end work per version: entry points,
  shader translation, PVRTC, IOSurface targets, ES 2.0 + Apple
  extensions. On 32-bit A5/A6 guests the UI stays on OpenGL ES (no
  Metal) and storage stays HFS+ (no APFS) — both are part of why 8/9/10
  on 32-bit is tractable at all.

## A6 (0.6) — the second SoC

Target: **iPhone 5 (n41ap, A6/S5L8950) on iOS 10.3.4 (14G61)**; siblings
iPhone 5c, iPad 4 (A6X). A6 is a *second* from-scratch SoC, not an A5
upgrade (new MMIO/clock tree, Swift cores, SGX543MP3/554-class GL scope,
Lightning-era companions), plus APFS rootfs handling in the preparer
(APFS arrives in 10.3). Open question: no firmware keys are published
for any iOS 10 release build on A6 — whether the final 32-bit boot
chain is unencrypted must be settled by inspecting a 10.3.4 IPSW's
KBAGs before any promise; fallback is checkm8 key extraction from a
physical device (A6 is checkm8-vulnerable) — a hardware errand, not a
download.

## Legacy engine integration (parallel track, feeds 0.2)

- Vendor the qemu-ios engine behind the Rust core: **the UI never links
  QEMU — a worker process hosts the engine**, one per running device
  (fault containment and the GPL boundary in one move; permissive-only
  crates stay out of the worker). Prepared `base/` + writable `state/`
  per library record; preparation is download → SHA-256 verify →
  prepare; ACPI graceful shutdown with the dirty-flag/snapshot path.
- Framebuffer view: raw texture, never tinted.
- iOS: Rust core over FFI under the SwiftUI shell; JIT prewarm
  activation — port Husk's allocator core, claim exactly the engine's
  tb-size at launch while the debugger is attached, hand QEMU TCG the
  split-W^X region, detach. TCI interpreter fallback when JIT is not
  enabled.

## Later, deliberately

A7 (S5L8960/8965 — note T7000 is the **A8**) adds ARMv8, IMG4/IM4M
personalisation, a Secure Enclave model, DART and a Rogue-class GPU
story. Research forks for A9/A13 prove the category; ChefKissInc's code
is GPL-3/AGPL-3 and is reference-reading only for this GPL-2.0 project.
Everything A7+ stays picker-tier "Future" until this is real work, not
a paragraph.

## Cross-cutting rules (every phase)

- Privacy as types: only a dedicated net crate may open sockets, and
  every network call needs a consent token only a UI button handler can
  mint — no background networking is expressible. Downloads: Apple hosts
  only, user tap, SHA-256 verified. Guest networking off by default.
  Per-device identity (serial/UDID/ECID/MAC) is randomly generated;
  host identity never reaches the guest. No telemetry, ever.
- Worker isolation per device; no auto-restart loop after 3 failed boots.
- Release gates: catalog verified against ipsw.me at generation time,
  SHA256SUMS on every asset, licence notices in every artifact, honest
  labels per `docs/COMPATIBILITY.md`, milestone evidence for stage moves.
