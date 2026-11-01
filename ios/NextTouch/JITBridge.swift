// NextTouch iOS — Copyright (c) 2026 c12n Apps
// SPDX-License-Identifier: GPL-2.0-or-later
//
// Swift bridge over the C JIT substrate in ios-jit/ (NextTouchJIT.h),
// which is adapted from Husk (GPL-2.0-or-later) — see ios-jit/README.md.
// Detection (csops / CS_DEBUGGED) is live in v0.1.0; the prewarm/allocator
// path is an explicitly inert skeleton until the engine phase.

import Foundation

enum JITStatus: Equatable {
    /// A debugger is attached (CS_DEBUGGED): the JIT path can be armed.
    case ready
    /// No debugger attached this launch: JIT is not enabled.
    case notEnabled
    /// The user chose the interpreter fallback for this launch.
    case interpreterFallback

    var title: String {
        switch self {
        case .ready: return "JIT ready"
        case .notEnabled: return "JIT not enabled"
        case .interpreterFallback: return "Interpreter fallback"
        }
    }
}

enum JITBridge {
    /// Poll csops for CS_DEBUGGED (implemented in NextTouchJIT.c).
    static func debuggerAttached() -> Bool {
        nt_jit_debugger_attached() != 0
    }

    static func currentStatus(userChoseInterpreter: Bool) -> JITStatus {
        if debuggerAttached() { return .ready }
        return userChoseInterpreter ? .interpreterFallback : .notEnabled
    }

    /// Attempt the one-shot prewarm. In v0.1.0 this returns
    /// NT_JIT_ERR_NOT_IMPLEMENTED by design — there is no engine code
    /// cache to receive the region yet. The JIT screen says exactly that;
    /// nothing in the app presents it as a working prewarm.
    static func prewarm(bytes: Int) -> Int32 {
        nt_jit_prewarm(bytes)
    }
}
