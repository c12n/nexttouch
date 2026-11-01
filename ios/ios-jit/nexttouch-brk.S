/*
 * nexttouch-brk.S — breakpoint RPC routines for the NextTouch JIT substrate
 *
 * Copyright (c) 2026 c12n Apps
 * SPDX-License-Identifier: GPL-2.0-or-later
 *
 * Ported from Husk's src/ios-jit/husk-brk.S
 * (https://github.com/Leviidev/Husk, GPL-2.0-or-later). Husk implements
 * this natively rather than linking a breakpoint-JIT framework (an
 * embedded framework carrying entitlements is rejected by AMFI at launch
 * on sideloaded builds); NextTouch does the same.
 *
 * Protocol: the attached debugger stops on `brk #0xf00d`; the command is
 * in x16, arguments in x0/x1, and the debugger writes the result back
 * into x0 before continuing the process.
 *   x16 = 1  CMD_PREPARE_REGION (addr in x0, len in x1 -> base in x0)
 *   x16 = 0  CMD_DETACH
 * The probe uses `brk #0x69`; the debugger answers with 0xE0000069 in x0.
 *
 * WARNING: with no debugger servicing the trap these instructions raise
 * SIGTRAP. Only call them behind the trap guard and after attach
 * detection (see NextTouchJIT.h). v0.1.0 compiles these routines in but
 * the app does not invoke them — the allocator that uses them lands with
 * the engine phase (ios-jit/README.md).
 */

#if defined(__arm64__)

    .text
    .align 2

    .globl _nt_brk_get_jit_mapping
    .p2align 2
_nt_brk_get_jit_mapping:
    mov x16, #1
    brk #0xf00d
    ret

    .globl _nt_brk_jit_detach
    .p2align 2
_nt_brk_jit_detach:
    mov x16, #0
    brk #0xf00d
    ret

    .globl _nt_brk_probe
    .p2align 2
_nt_brk_probe:
    brk #0x69
    ret

#endif
