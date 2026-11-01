// NextTouch iOS — Copyright (c) 2026 c12n Apps
// SPDX-License-Identifier: GPL-2.0-or-later

import SwiftUI

struct LibraryView: View {
    @EnvironmentObject var state: AppState

    var body: some View {
        NavigationStack {
            List {
                if state.library.isEmpty {
                    Text("No devices yet. Add one from the Device picker.")
                        .foregroundStyle(.secondary)
                }
                ForEach(state.library) { item in
                    VStack(alignment: .leading, spacing: 2) {
                        Text(item.name).fontWeight(.semibold)
                        Text("\(item.deviceIdentifier) · iOS \(item.firmwareVersion)")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                        Text("local id \(String(item.id.prefix(16)))")
                            .font(.caption2)
                            .foregroundStyle(.secondary)
                        if item.dirty {
                            Text("dirty — snapshot restore offered on launch")
                                .font(.caption)
                                .foregroundStyle(.orange)
                        }
                    }
                }
                .onDelete { offsets in
                    for index in offsets {
                        state.removeFromLibrary(state.library[index])
                    }
                }
            }
            .navigationTitle("Library")
        }
    }
}
