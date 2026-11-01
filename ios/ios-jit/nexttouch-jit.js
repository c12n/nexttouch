// nexttouch-jit.js — debugger-side JIT dispatch script (SKELETON, v0.1.0)
//
// Copyright (c) 2026 c12n Apps
// SPDX-License-Identifier: GPL-2.0-or-later
//
// Lineage: the production version of this script follows the dispatch
// loop in Husk's src/app/Husk/Resources/husk-jit.js (GPL-2.0-or-later),
// itself adapted from StikDebug's universal script (StikDebug is
// AGPL-3.0 and stays a separate install — it is never linked into
// NextTouch; see THIRD_PARTY_NOTICES.md).
//
// What the production script does (documented here, implemented with the
// engine phase — v0.1.0 intentionally ships this skeleton only, because
// the app side does not issue RPCs yet):
//
//   1. Attach to the app process over the gdb-remote protocol
//      (vAttach;<pid>), then loop on continue (`c`).
//   2. On each stop, read thread / pc / x16. A stop at `brk #0xf00d` is a
//      JIT RPC:
//        x16 = 1 (CMD_PREPARE_REGION): allocate an RX region of x1 bytes
//          in the tracee (debugserver `_M<size>,rx`), then write one byte
//          (`M<addr>,1:69`) into EVERY 16 KiB page — under TXM that
//          per-page debugger write is what makes the pages executable.
//          Write the region base back into x0.
//        x16 = 0 (CMD_DETACH): detach; allocation was one-shot.
//      A stop at `brk #0x69` is the liveness probe: write 0xE0000069
//      into x0 and continue.
//   3. Any other stop is forwarded to the user/debugger UI untouched.
//
// The packet count grows linearly with region size (a 512 MiB region is
// 32,768 page-write packets), which is why NextTouch will prewarm exactly
// the engine's tb-size rather than Husk's two-consumer 512 MiB.
//
// TODO(engine-phase): implement the dispatch loop against the debugger
// hosts NextTouch supports (external StikDebug URL-scheme hand-off first;
// a built-in helper is a later phase), mirroring husk-jit.js.
