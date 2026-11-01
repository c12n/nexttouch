// NextTouch iOS — Copyright (c) 2026 c12n Apps
// SPDX-License-Identifier: GPL-2.0-or-later

import SwiftUI

struct AboutView: View {
    @EnvironmentObject var state: AppState

    var body: some View {
        NavigationStack {
            List {
                Section {
                    LabeledContent("Version", value: "0.1.0")
                    LabeledContent("Catalog", value: "v\(state.catalog.catalogVersion) · \(state.catalog.generatedAtUTC)")
                    Text("NextTouch extends the qemu-ios emulator family (LightTouch / iOS-Emulator) toward iOS 8, 9 and 10. v0.1.0 ships the shared catalog, picker, library and settings on Windows, Android and iOS. The A5 machine (iPhone 4S, iOS 8.4.1) is the first engine target; no iOS 8/9/10 firmware boots yet and the picker labels it that way.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                }
                Section("Privacy") {
                    Text("No telemetry, no analytics, no crash reporting, no accounts, no third-party SDKs. This build contains no tracking (see PrivacyInfo.xcprivacy) and makes no network calls.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                }
                Section("Licences") {
                    Text("NextTouch — Copyright (c) 2026 c12n Apps. GPL-2.0-or-later.")
                        .font(.footnote)
                    Text("Built on: LightTouch (GPL-2.0-or-later), mada-02/iOS-Emulator (GPL-2.0-or-later), qemu-ios / QEMU (GPL-2.0). JIT substrate adapted from Husk (GPL-2.0-or-later). Full notices ship in THIRD_PARTY_NOTICES.md with every release.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                    Text("iPhone, iPad and iPod touch are trademarks of Apple Inc. This project is not affiliated with Apple and does not distribute Apple firmware.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                }
            }
            .navigationTitle("About")
        }
    }
}
