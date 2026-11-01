/*
 * NextTouchJIT.h — NextTouch iOS JIT substrate (public C interface)
 *
 * Copyright (c) 2026 c12n Apps
 * SPDX-License-Identifier: GPL-2.0-or-later
 *
 * Adapted from Husk (https://github.com/Leviidev/Husk), GPL-2.0-or-later:
 * the breakpoint-RPC protocol shape and the csops/CS_DEBUGGED attach
 * detection follow Husk's src/ios-jit/husk-ios-jit.h and
 * src/app/Husk/JITBootstrap.swift. The allocator core in Husk's
 * husk-ios-jit.c is itself derived from AetherPS4's JIT allocator
 * (GPL family); see THIRD_PARTY_NOTICES.md.
 *
 * v0.1.0: detection is live; the region allocator is an explicitly inert
 * skeleton (NT_JIT_ERR_NOT_IMPLEMENTED) that activates with the engine
 * phase. See ios-jit/README.md — do not mistake the skeleton for a
 * working prewarm.
 */

#ifndef NEXTTOUCH_JIT_H
#define NEXTTOUCH_JIT_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Result codes. */
#define NT_JIT_OK 0
#define NT_JIT_ERR_NO_DEBUGGER (-1)       /* no debugger attached; RPC unserviced */
#define NT_JIT_ERR_NOT_IMPLEMENTED (-2)   /* v0.1.0 skeleton; engine phase pending */
#define NT_JIT_ERR_PROBE_FAILED (-3)      /* region allocated but not executable */

/* Three-state JIT status shown by the app's JIT screen. */
typedef enum {
    /* A debugger is attached (CS_DEBUGGED set): the JIT path can be armed
     * (engine phase: prewarm runs in this state). */
    NT_JIT_STATUS_READY = 0,
    /* No debugger attached: JIT is not enabled for this launch. */
    NT_JIT_STATUS_NOT_ENABLED = 1,
    /* Fallback: no JIT — the engine phase runs QEMU's TCI interpreter. */
    NT_JIT_STATUS_INTERPRETER = 2,
} nt_jit_status_t;

/* Poll csops(CS_OPS_STATUS) for CS_DEBUGGED. Returns 1 when a debugger is
 * attached to this process, 0 otherwise. This is the ONLY attach signal
 * the app trusts (Husk JITBootstrap.isDebuggerAttached does the same). */
int nt_jit_debugger_attached(void);

/* Current three-state status:
 *  - READY when a debugger is attached,
 *  - INTERPRETER when the caller has chosen the interpreter fallback for
 *    this launch (nt_jit_use_interpreter_fallback was called),
 *  - NOT_ENABLED otherwise. */
nt_jit_status_t nt_jit_current_status(void);

/* Record the user's choice to run in interpreter mode for this launch. */
void nt_jit_use_interpreter_fallback(void);

/*
 * Breakpoint RPC (implemented in nexttouch-brk.S, ported from Husk's
 * husk-brk.S). These trap into the attached debugger; with no debugger
 * servicing the trap they raise SIGTRAP, so callers must only invoke them
 * behind the trap guard and after nt_jit_debugger_attached() == 1.
 *
 *   nt_brk_get_jit_mapping(addr, len)  x16 = 1  CMD_PREPARE_REGION:
 *       the debugger allocates an RX region at/near addr of len bytes,
 *       writes one byte into every 16 KiB page, and writes the resulting
 *       base address back into x0.
 *   nt_brk_jit_detach()                x16 = 0  CMD_DETACH
 *   nt_brk_probe()                     brk #0x69: liveness probe; the
 *       debugger answers by writing 0xE0000069 into x0.
 */
void *nt_brk_get_jit_mapping(void *addr, size_t len);
void nt_brk_jit_detach(void);
uint64_t nt_brk_probe(void);

/*
 * Prewarm the JIT code-cache region (one-shot).
 *
 * TODO(engine-phase): port the allocator core from Husk's
 * src/ios-jit/husk-ios-jit.c — request the RX region via
 * nt_brk_get_jit_mapping, vm_remap a writable alias, run the
 * two-instruction self-test, detach. NextTouch prewarms exactly the
 * engine's tb-size (one consumer), not Husk's 512 MiB two-consumer split.
 *
 * v0.1.0: always returns NT_JIT_ERR_NOT_IMPLEMENTED. There is no QEMU
 * code cache to receive the region yet; a live prewarm now would be fake.
 */
int nt_jit_prewarm(size_t bytes);

#ifdef __cplusplus
}
#endif

#endif /* NEXTTOUCH_JIT_H */
