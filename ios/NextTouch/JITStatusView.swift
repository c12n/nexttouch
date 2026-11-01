// NextTouch iOS — Copyright (c) 2026 c12n Apps
// SPDX-License-Identifier: GPL-2.0-or-later
//
// JIT status screen. The mechanism follows Husk (see ios-jit/README.md):
// an attached debugger services a breakpoint RPC and touches every
// 16 KiB page of the code-cache region; allocation is one-shot at launch.
// v0.1.0 implements detection (csops / CS_DEBUGGED) and reports the
// three states honestly; the prewarm -> QEMU handoff activates with the
// engine phase and this screen says so.

import SwiftUI

struct JITStatusView: View {
    @State private var attached = JITBridge.debuggerAttached()
    @State private var choseInterpreter = false

    private var status: JITStatus {
        JITBridge.currentStatus(userChoseInterpreter: choseInterpreter)
    }

    var body: some View {
        NavigationStack {
            List {
                Section("Status this launch") {
                    HStack {
                        Image(systemName: icon)
                            .foregroundStyle(color)
                        Text(status.title).font(.headline)
                    }
                    Text(explanation)
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                }
                Section("How JIT works here") {
                    Text("NextTouch follows the Husk method: sign with a development certificate (get-task-allow), let a debugger attach, and the app asks it — over a breakpoint RPC — to make the emulator's code cache executable, one 16 KiB page at a time. The region is claimed once, at launch, then the debugger detaches.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                    Text("JIT is per-launch: it must be re-enabled each time the app starts. A distribution-signed copy can never be JIT-enabled.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                }
                Section("Engine phase") {
                    Text("In v0.1.0 there is no engine and no code cache yet, so the prewarm step is deliberately inert (the C substrate returns “not implemented”) — nothing here fakes a JIT allocation. Prewarm activates with the engine integration; see the roadmap in the repository.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                }
                Section {
                    Button("Re-check") {
                        attached = JITBridge.debuggerAttached()
                    }
                    Button("Use interpreter fallback this launch") {
                        choseInterpreter = true
                        _ = JITBridge.prewarm(bytes: 0) // inert in v0.1.0; documents intent
                    }
                    .disabled(status != .notEnabled)
                }
            }
            .navigationTitle("JIT")
            .onAppear { attached = JITBridge.debuggerAttached() }
        }
    }

    private var icon: String {
        switch status {
        case .ready: return "bolt.fill"
        case .notEnabled: return "bolt.slash"
        case .interpreterFallback: return "tortoise"
        }
    }

    private var color: Color {
        switch status {
        case .ready: return .green
        case .notEnabled: return .orange
        case .interpreterFallback: return .blue
        }
    }

    private var explanation: String {
        switch status {
        case .ready:
            return "A debugger is attached to NextTouch (CS_DEBUGGED is set). When the engine lands, the code cache is prewarmed in exactly this state."
        case .notEnabled:
            return "No debugger is attached, so JIT is not enabled for this launch. Enable it with your usual development-certificate JIT route, then re-check — or use the interpreter fallback (much slower)."
        case .interpreterFallback:
            return "You chose the interpreter fallback for this launch: the engine phase will run QEMU's TCI interpreter instead of JIT code. Relaunch and enable JIT for full speed."
        }
    }
}
