// NextTouch iOS — Copyright (c) 2026 c12n Apps
// SPDX-License-Identifier: GPL-2.0-or-later
//
// App state: catalog, library (app-private JSON, random local ids,
// mirroring the Rust core's library.json format) and the theme
// preference. Settings are stored as JSON in the app container — the
// same "settings as a file in the data folder" rule as the other shells.

import Foundation
import SwiftUI

enum ThemeChoice: String, CaseIterable {
    case system
    case light
    case dark

    var label: String {
        switch self {
        case .system: return "Follow system"
        case .light: return "Light"
        case .dark: return "Dark"
        }
    }

    var colorScheme: ColorScheme? {
        switch self {
        case .system: return nil
        case .light: return .light
        case .dark: return .dark
        }
    }
}

@MainActor
final class AppState: ObservableObject {
    @Published var catalog: Catalog
    @Published var library: [LibraryItem] = []
    @AppStorage("nt_theme") var themeRaw: String = ThemeChoice.system.rawValue

    var theme: ThemeChoice {
        get { ThemeChoice(rawValue: themeRaw) ?? .system }
        set { themeRaw = newValue.rawValue }
    }

    private var libraryURL: URL {
        let base = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
        return base.appendingPathComponent("NextTouch/library.json")
    }

    init() {
        catalog = Catalog.loadBundled()
        loadLibrary()
    }

    func loadLibrary() {
        guard let data = try? Data(contentsOf: libraryURL),
              let items = try? JSONDecoder().decode([LibraryItem].self, from: data)
        else {
            library = []
            return
        }
        library = items
    }

    func saveLibrary() {
        let url = libraryURL
        try? FileManager.default.createDirectory(
            at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
        if let data = try? JSONEncoder().encode(library) {
            try? data.write(to: url, options: [.atomic])
        }
    }

    func addToLibrary(device: Device, firmware: FirmwareEntry) {
        let item = LibraryItem(
            id: UUID().uuidString.replacingOccurrences(of: "-", with: "").lowercased(),
            name: "\(device.name) (\(firmware.version ?? ""))",
            deviceIdentifier: device.identifier,
            firmwareID: firmware.id ?? "",
            firmwareVersion: firmware.version ?? ""
        )
        library.append(item)
        saveLibrary()
    }

    func removeFromLibrary(_ item: LibraryItem) {
        library.removeAll { $0.id == item.id }
        saveLibrary()
    }

    /// Settings > "Delete all NextTouch data": removes the whole
    /// app-private NextTouch folder and the stored preferences.
    func deleteAllData() {
        let base = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
        try? FileManager.default.removeItem(at: base.appendingPathComponent("NextTouch"))
        UserDefaults.standard.removeObject(forKey: "nt_theme")
        library = []
    }
}
