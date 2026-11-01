/*
 * NextTouchJIT.c — NextTouch iOS JIT substrate
 *
 * Copyright (c) 2026 c12n Apps
 * SPDX-License-Identifier: GPL-2.0-or-later
 *
 * Adapted from Husk (https://github.com/Leviidev/Husk), GPL-2.0-or-later:
 * attach detection follows Husk's JITBootstrap (csops CS_OPS_STATUS /
 * CS_DEBUGGED) and the RPC command split follows src/ios-jit/husk-brk.S.
 * See ios-jit/README.md for the full mechanism and THIRD_PARTY_NOTICES.md
 * for attribution.
 *
 * v0.1.0 scope, honestly: detection + status are live. The region
 * allocator (nt_jit_prewarm) is a marked skeleton — the engine phase
 * ports Husk's husk-ios-jit.c allocator core. Nothing here pretends a
 * prewarm happened.
 */

#include "NextTouchJIT.h"

#include <stdint.h>
#include <sys/syscall.h>
#include <unistd.h>

/*
 * csops is not declared in the public iOS SDK headers; declare it the way
 * Husk's JITBootstrap does (there bound via @_silgen_name("csops")).
 *
 * Note (from Husk's husk-ios-jit.c): on jailbroken systems a libc hook
 * may hide CS_DEBUGGED from the csops library call; Husk's engine-side
 * code uses a raw syscall(SYS_csops) there. v0.1.0's detection is for the
 * development-signed sideload route (no jailbreak hooks in play), so the
 * plain call is correct here; the raw-syscall variant lands with the
 * engine-phase port.
 */
extern int csops(pid_t pid, unsigned int ops, void *useraddr, size_t usersize);

#define NT_CS_OPS_STATUS 0
#define NT_CS_DEBUGGED 0x10000000u

int nt_jit_debugger_attached(void) {
    uint32_t flags = 0;
    if (csops(getpid(), NT_CS_OPS_STATUS, &flags, sizeof(flags)) != 0) {
        return 0;
    }
    return (flags & NT_CS_DEBUGGED) != 0 ? 1 : 0;
}

static int g_interpreter_fallback = 0;

void nt_jit_use_interpreter_fallback(void) {
    g_interpreter_fallback = 1;
}

nt_jit_status_t nt_jit_current_status(void) {
    if (nt_jit_debugger_attached()) {
        return NT_JIT_STATUS_READY;
    }
    if (g_interpreter_fallback) {
        return NT_JIT_STATUS_INTERPRETER;
    }
    return NT_JIT_STATUS_NOT_ENABLED;
}

int nt_jit_prewarm(size_t bytes) {
    (void)bytes;
    /*
     * TODO(engine-phase): faithful port of Husk's allocator core
     * (src/ios-jit/husk-ios-jit.c, derived from AetherPS4's allocator):
     *   1. install the SIGTRAP/SIGBUS trap guard (unserviced brk -> error,
     *      never process death),
     *   2. nt_brk_probe() to confirm the debugger services the RPC,
     *   3. nt_brk_get_jit_mapping(NULL, bytes) for the RX region,
     *   4. vm_remap() a writable alias of the same pages,
     *   5. self-test: write + execute a two-instruction function,
     *   6. nt_brk_jit_detach() — allocation is one-shot.
     * Then hand the split-W^X region to QEMU TCG via the region.c
     * allocator re-applied to the qemu-ios fork. Until the engine exists
     * there is nothing to hand the region to, so this stays inert.
     */
    return NT_JIT_ERR_NOT_IMPLEMENTED;
}
