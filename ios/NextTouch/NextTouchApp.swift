// NextTouch iOS — Copyright (c) 2026 c12n Apps
// SPDX-License-Identifier: GPL-2.0-or-later

import SwiftUI

@main
struct NextTouchApp: App {
    @StateObject private var state = AppState()

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(state)
                .preferredColorScheme(state.theme.colorScheme)
        }
    }
}
