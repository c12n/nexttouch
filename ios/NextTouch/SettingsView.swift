// NextTouch iOS — Copyright (c) 2026 c12n Apps
// SPDX-License-Identifier: GPL-2.0-or-later

import SwiftUI

struct SettingsView: View {
    @EnvironmentObject var state: AppState
    @State private var confirmWipe = false
    @State private var wiped = false

    var body: some View {
        NavigationStack {
            List {
                Section("Theme") {
                    Picker("Theme", selection: themeBinding) {
                        ForEach(ThemeChoice.allCases, id: \.self) { choice in
                            Text(choice.label).tag(choice)
                        }
                    }
                    .pickerStyle(.inline)
                    Text("Applies to the app chrome only — an emulated screen is never tinted.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                }
                Section("Privacy") {
                    Text("All data is app-private and local. No telemetry, no accounts, no update checks. The only planned network path is downloading an IPSW from Apple, on your tap, verified by SHA-256.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                    Text("Guest networking: off (default; takes effect in the engine phase).")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                }
                Section("Danger zone") {
                    Button("Delete all NextTouch data…", role: .destructive) {
                        confirmWipe = true
                    }
                    if wiped {
                        Text("All NextTouch data deleted.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                    }
                }
            }
            .navigationTitle("Settings")
            .alert("Delete all NextTouch data?", isPresented: $confirmWipe) {
                Button("Delete everything", role: .destructive) {
                    state.deleteAllData()
                    wiped = true
                }
                Button("Cancel", role: .cancel) {}
            } message: {
                Text("This permanently deletes the app-private NextTouch folder — settings, library and all device data.")
            }
        }
    }

    private var themeBinding: Binding<ThemeChoice> {
        Binding(
            get: { state.theme },
            set: { state.theme = $0; state.objectWillChange.send() }
        )
    }
}
