// NextTouch iOS — Copyright (c) 2026 c12n Apps
// SPDX-License-Identifier: GPL-2.0-or-later

import SwiftUI

struct ContentView: View {
    var body: some View {
        TabView {
            PickerView()
                .tabItem { Label("Picker", systemImage: "iphone") }
            LibraryView()
                .tabItem { Label("Library", systemImage: "books.vertical") }
            JITStatusView()
                .tabItem { Label("JIT", systemImage: "bolt") }
            SettingsView()
                .tabItem { Label("Settings", systemImage: "gearshape") }
            AboutView()
                .tabItem { Label("About", systemImage: "info.circle") }
        }
    }
}
