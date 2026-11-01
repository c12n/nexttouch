// NextTouch iOS — Copyright (c) 2026 c12n Apps
// SPDX-License-Identifier: GPL-2.0-or-later

import SwiftUI

struct StatusChip: View {
    let status: FirmwareStatus

    var body: some View {
        Text(status.label)
            .font(.caption2)
            .fontWeight(.semibold)
            .padding(.horizontal, 6)
            .padding(.vertical, 2)
            .background(color.opacity(0.2))
            .foregroundStyle(color)
            .clipShape(Capsule())
    }

    private var color: Color {
        switch status {
        case .tested: return .green
        case .testedSlow: return .orange
        case .experimental: return .blue
        case .notBootingYet: return .red
        }
    }
}

struct PickerView: View {
    @EnvironmentObject var state: AppState
    @State private var query = ""

    var body: some View {
        NavigationStack {
            List {
                Section {
                    Text("Legacy firmwares (iPhone OS 1 – iOS 7) carry the baseline engine's real status. Every iOS 8 / 9 / 10 firmware is labelled “Not booting yet”: the A5/A6 engine work is in development and v0.1.0 does not boot them.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                }
                if query.isEmpty {
                    Section("Quick start (development defaults)") {
                        ForEach(state.catalog.quickStart, id: \.self) { key in
                            let parts = key.split(separator: ":").map(String.init)
                            if parts.count == 2,
                               let dev = state.catalog.device(parts[0]),
                               let fw = dev.firmwares.first(where: { $0.version == parts[1] }) {
                                NavigationLink {
                                    DeviceDetailView(device: dev, firmware: fw)
                                } label: {
                                    Text("\(dev.name) — iOS \(parts[1])")
                                }
                            }
                        }
                    }
                }
                Section("Devices") {
                    ForEach(state.catalog.search(query)) { dev in
                        NavigationLink {
                            FirmwareListView(device: dev)
                        } label: {
                            VStack(alignment: .leading, spacing: 2) {
                                Text(dev.name).fontWeight(.semibold)
                                Text("\(dev.identifier) · \(dev.soc) (\(dev.socChip)) · \(dev.arch)")
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                                Text(dev.tier.label)
                                    .font(.caption2)
                                    .foregroundStyle(.secondary)
                            }
                        }
                    }
                }
            }
            .navigationTitle("Device picker")
            .searchable(text: $query, prompt: "Search devices")
        }
    }
}

struct FirmwareListView: View {
    let device: Device

    var body: some View {
        List {
            Section {
                LabeledContent("Identifier", value: device.identifier)
                LabeledContent("Board", value: device.board)
                LabeledContent("SoC", value: "\(device.soc) (\(device.socChip))")
                LabeledContent("Architecture", value: device.arch)
                LabeledContent("Display", value: "\(device.displayPx[0])×\(device.displayPx[1])")
                LabeledContent("RAM", value: "\(device.ramMB) MB")
                LabeledContent("Tier", value: device.tier.label)
            }
            Section("Firmwares") {
                ForEach(device.firmwares, id: \.listID) { fw in
                    NavigationLink {
                        DeviceDetailView(device: device, firmware: fw)
                    } label: {
                        HStack {
                            VStack(alignment: .leading, spacing: 2) {
                                Text("\(fw.osName ?? "iOS") \(fw.version ?? "—") (\(fw.build ?? "—"))")
                                if let file = fw.ipswFilename {
                                    Text(file).font(.caption2).foregroundStyle(.secondary)
                                }
                            }
                            Spacer()
                            StatusChip(status: fw.status)
                        }
                    }
                    .disabled(fw.id == nil)
                }
            }
        }
        .navigationTitle(device.name)
    }
}

struct DeviceDetailView: View {
    @EnvironmentObject var state: AppState
    let device: Device
    let firmware: FirmwareEntry
    @State private var added = false

    var body: some View {
        List {
            Section {
                HStack {
                    Text("\(firmware.osName ?? "iOS") \(firmware.version ?? "—")")
                        .font(.headline)
                    Spacer()
                    StatusChip(status: firmware.status)
                }
                LabeledContent("Build", value: firmware.build ?? "—")
                LabeledContent("Device", value: "\(device.name) (\(device.identifier))")
                LabeledContent("Board / SoC", value: "\(device.board) · \(device.socChip)")
                if let file = firmware.ipswFilename {
                    LabeledContent("IPSW", value: file)
                }
                if let sha = firmware.sha256 {
                    VStack(alignment: .leading, spacing: 2) {
                        Text("SHA-256").foregroundStyle(.secondary)
                        Text(sha).font(.system(.caption2, design: .monospaced))
                    }
                } else {
                    LabeledContent("SHA-256", value: "Not available (\(firmware.sha256Note ?? "no reason recorded"))")
                }
            }
            Section {
                // The emulated-screen area. In v0.1.0 it tells the truth
                // instead of faking a display (engine phase fills this).
                Text(firmware.status.isRunnable
                     ? "Engine not integrated in v0.1.0. This firmware runs on the baseline engine in LightTouch today; NextTouch's engine integration is the next roadmap phase. No screen is faked here."
                     : "Not booting yet. The engine has no machine for this SoC yet. v0.1.0 ships the picker, catalog and roadmap — no boot is claimed for this firmware.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
            }
            Section {
                Button(added ? "Added to library" : "Add to library") {
                    state.addToLibrary(device: device, firmware: firmware)
                    added = true
                }
                .disabled(added)
            }
        }
        .navigationTitle(device.name)
    }
}
