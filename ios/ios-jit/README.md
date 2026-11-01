# NextTouch iOS JIT — Copyright (c) 2026 c12n Apps
SPDX-License-Identifier: GPL-2.0-or-later

This directory holds NextTouch's iOS JIT substrate, adapted from
**Husk** (https://github.com/Leviidev/Husk, GPL-2.0-or-later) — see
`THIRD_PARTY_NOTICES.md` and the header of each file. The mechanism, in
Husk's design (which NextTouch follows deliberately):

- An app cannot grant itself JIT on modern iOS. On TXM systems (iOS 26 on
  A14+, iOS 27 almost everywhere) only an **attached debugger** can create
  executable memory.
- The app therefore arranges for a debugger to attach (StikDebug, a
  built-in StikJIT-style helper, TrollStore, or a jailbreak — the attach
  sources are external to this directory), then talks to that debugger
  over a **breakpoint RPC**: `brk #0xf00d` with a command number in `x16`.
- The debugger allocates an RX region inside the app and writes one byte
  into **every 16 KiB page** of it; that per-page debugger write is what
  makes the pages executable under TXM.
- The app then `vm_remap`s a writable alias of the same pages, self-tests
  by executing a two-instruction function in the region, and (in the
  engine phase) hands QEMU TCG the split-W^X dual mapping it wants.
- Allocation is **one-shot**: the code-cache region is prewarmed at launch
  while the debugger attachment is fresh, then the debugger is detached
  and cannot be asked again.
- Attach detection is polling `csops(CS_OPS_STATUS)` for `CS_DEBUGGED`.

## v0.1.0 status — read this before believing anything here

Compiled into the v0.1.0 IPA and functional:

- `NextTouchJIT.h` / parts of `NextTouchJIT.c`: debugger-attach detection
  via `csops`, and the three-state JIT status the app's JIT screen shows
  (`nt_jit_status`).
- `nexttouch-brk.S`: the three breakpoint-RPC routines (prepare-region,
  detach, probe), ported from Husk's `husk-brk.S`.

Present but **deliberately inert** in v0.1.0 (returns
`NT_JIT_ERR_NOT_IMPLEMENTED`, clearly marked TODO):

- The region allocator / prewarm / `vm_remap` alias in `NextTouchJIT.c`.
  There is no engine and no QEMU code cache to hand the region to yet, so
  shipping a live prewarm now would be theatre. It activates with the
  engine phase per `docs/ROADMAP.md`, following the file-by-file port
  list in the project's Husk audit (allocator core from Husk's
  `husk-ios-jit.c`, itself derived from AetherPS4's allocator, plus the
  `tcg/region.c` split-W^X allocator re-applied by hand to the qemu-ios
  fork's older QEMU base).
- `nexttouch-jit.js`: the debugger-side dispatch script is a documented
  skeleton in v0.1.0. The production script is bundled with the engine
  phase; its lineage (StikDebug's script, AGPL-3.0) is recorded in
  `THIRD_PARTY_NOTICES.md` — the debugger stays a separate component.

Husk's certificate-import "play without JIT" AOT path is **not** ported:
it code-signs translated guest libraries and offers QEMU TCG nothing.
NextTouch's no-JIT fallback is QEMU's TCI interpreter, per the roadmap.
