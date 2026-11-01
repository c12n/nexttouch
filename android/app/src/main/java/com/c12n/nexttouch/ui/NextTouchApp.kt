// NextTouch Android UI — Copyright (c) 2026 c12n Apps
// SPDX-License-Identifier: GPL-2.0-or-later
//
// v0.1.0 scope (same honesty contract as the other shells): picker +
// library + settings + about over the shared catalog. The engine is not
// integrated yet; every iOS 8/9/10 firmware is labelled "Not booting yet".
// This app contains no networking code in v0.1.0 — the only future network
// path is an IPSW download from Apple hosts on explicit user tap.

package com.c12n.nexttouch.ui

import android.content.Context
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.c12n.nexttouch.model.Catalog
import com.c12n.nexttouch.model.Device
import com.c12n.nexttouch.model.FirmwareEntry
import kotlinx.serialization.json.Json
import java.io.File
import java.security.SecureRandom

private enum class ThemeChoice(val key: String, val label: String) {
    SYSTEM("system", "Follow system"),
    LIGHT("light", "Light"),
    DARK("dark", "Dark"),
}

private enum class Tab(val label: String) {
    PICKER("Picker"), LIBRARY("Library"), SETTINGS("Settings"), ABOUT("About"),
}

private data class LibraryItem(
    val id: String,
    val name: String,
    val deviceIdentifier: String,
    val firmwareId: String,
    val firmwareVersion: String,
)

private fun randomIdHex(): String {
    val bytes = ByteArray(16)
    SecureRandom().nextBytes(bytes)
    return bytes.joinToString("") { "%02x".format(it) }
}

private fun loadCatalog(context: Context): Catalog {
    val text = context.assets.open("catalog.json").bufferedReader().use { it.readText() }
    return Json { ignoreUnknownKeys = true }.decodeFromString(Catalog.serializer(), text)
}

@Composable
fun NextTouchApp() {
    val context = LocalContext.current
    val prefs = remember { context.getSharedPreferences("nexttouch", Context.MODE_PRIVATE) }

    // The engine phase moves settings into the app-private JSON file like
    // the Rust core; v0.1.0 keeps the theme choice in private preferences,
    // which are wiped by "Delete all data" below.
    var themeChoice by remember {
        mutableStateOf(ThemeChoice.entries.firstOrNull { it.key == prefs.getString("theme", "system") } ?: ThemeChoice.SYSTEM)
    }
    val dark = when (themeChoice) {
        ThemeChoice.SYSTEM -> isSystemInDarkTheme()
        ThemeChoice.LIGHT -> false
        ThemeChoice.DARK -> true
    }
    val colorScheme = when {
        // Material 3 dynamic colour where the platform offers it (API 31+),
        // always honouring the explicit light/dark/follow-system choice.
        android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.S ->
            if (dark) dynamicDarkColorScheme(context) else dynamicLightColorScheme(context)
        else -> if (dark) darkColorScheme() else lightColorScheme()
    }

    val catalog = remember { loadCatalog(context) }
    var tab by remember { mutableStateOf(Tab.PICKER) }
    var selectedDevice by remember { mutableStateOf<Device?>(null) }
    var selectedFirmware by remember { mutableStateOf<FirmwareEntry?>(null) }
    val library = remember {
        mutableStateListOf<LibraryItem>().apply {
            val raw = prefs.getString("library_ids", "") ?: ""
            raw.split(";").filter { it.isNotBlank() }.forEach { row ->
                val p = row.split("|")
                if (p.size == 5) add(LibraryItem(p[0], p[1], p[2], p[3], p[4]))
            }
        }
    }
    fun persistLibrary() {
        val raw = library.joinToString(";") {
            listOf(it.id, it.name, it.deviceIdentifier, it.firmwareId, it.firmwareVersion).joinToString("|")
        }
        prefs.edit().putString("library_ids", raw).apply()
    }

    MaterialTheme(colorScheme = colorScheme) {
        Scaffold(
            bottomBar = {
                NavigationBar {
                    Tab.entries.forEach { t ->
                        NavigationBarItem(
                            selected = tab == t,
                            onClick = { tab = t },
                            label = { Text(t.label) },
                            icon = { },
                        )
                    }
                }
            },
        ) { padding ->
            Surface(modifier = Modifier.fillMaxSize().padding(padding)) {
                when {
                    selectedDevice != null && selectedFirmware != null && tab == Tab.PICKER -> {
                        DeviceScreen(
                            device = selectedDevice!!,
                            firmware = selectedFirmware!!,
                            onBack = { selectedFirmware = null },
                            onAddToLibrary = {
                                library.add(
                                    LibraryItem(
                                        id = randomIdHex(),
                                        name = "${selectedDevice!!.name} (${selectedFirmware!!.version ?: ""})",
                                        deviceIdentifier = selectedDevice!!.identifier,
                                        firmwareId = selectedFirmware!!.id ?: "",
                                        firmwareVersion = selectedFirmware!!.version ?: "",
                                    ),
                                )
                                persistLibrary()
                                selectedFirmware = null
                                tab = Tab.LIBRARY
                            },
                        )
                    }
                    tab == Tab.PICKER -> PickerScreen(
                        catalog = catalog,
                        onSelectFirmware = { d, f ->
                            selectedDevice = d
                            selectedFirmware = f
                        },
                    )
                    tab == Tab.LIBRARY -> LibraryScreen(
                        items = library,
                        onRemove = { item ->
                            library.remove(item)
                            persistLibrary()
                        },
                    )
                    tab == Tab.SETTINGS -> SettingsScreen(
                        themeChoice = themeChoice,
                        onThemeChange = {
                            themeChoice = it
                            prefs.edit().putString("theme", it.key).apply()
                        },
                        filesDir = context.filesDir,
                        onWipe = {
                            library.clear()
                            persistLibrary()
                            context.filesDir.listFiles()?.forEach { it.deleteRecursively() }
                        },
                    )
                    else -> AboutScreen()
                }
            }
        }
    }
}

@Composable
private fun StatusChip(status: com.c12n.nexttouch.model.FirmwareStatus) {
    Surface(
        color = when (status) {
            com.c12n.nexttouch.model.FirmwareStatus.TESTED -> MaterialTheme.colorScheme.primaryContainer
            com.c12n.nexttouch.model.FirmwareStatus.TESTED_SLOW -> MaterialTheme.colorScheme.tertiaryContainer
            com.c12n.nexttouch.model.FirmwareStatus.EXPERIMENTAL -> MaterialTheme.colorScheme.secondaryContainer
            com.c12n.nexttouch.model.FirmwareStatus.NOT_BOOTING_YET -> MaterialTheme.colorScheme.errorContainer
        },
        shape = MaterialTheme.shapes.small,
    ) {
        Text(
            status.label,
            modifier = Modifier.padding(horizontal = 8.dp, vertical = 2.dp),
            style = MaterialTheme.typography.labelSmall,
        )
    }
}

@Composable
private fun PickerScreen(catalog: Catalog, onSelectFirmware: (Device, FirmwareEntry) -> Unit) {
    var query by remember { mutableStateOf("") }
    var expanded by remember { mutableStateOf<String?>(null) }
    LazyColumn(
        modifier = Modifier.fillMaxSize().padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        item {
            Text("Device picker", style = MaterialTheme.typography.headlineSmall, fontWeight = FontWeight.Bold)
            Text(
                "Legacy firmwares (iPhone OS 1 – iOS 7) carry the baseline engine's real status. " +
                    "Every iOS 8 / 9 / 10 firmware is labelled “Not booting yet”: the A5/A6 engine " +
                    "work is in development and v0.1.0 does not boot them.",
            )
            Spacer(Modifier.height(8.dp))
            OutlinedTextField(
                value = query,
                onValueChange = { query = it },
                label = { Text("Search devices") },
                modifier = Modifier.fillMaxWidth(),
            )
            Spacer(Modifier.height(8.dp))
            Text("Quick start (development defaults)", fontWeight = FontWeight.Bold)
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                catalog.quickStart.forEach { key ->
                    val parts = key.split(":")
                    val dev = catalog.device(parts[0])
                    val fw = dev?.firmwares?.firstOrNull { it.version == parts.getOrNull(1) }
                    if (dev != null && fw != null) {
                        OutlinedButton(onClick = { onSelectFirmware(dev, fw) }) {
                            Text("${dev.name} · ${fw.version}")
                        }
                    }
                }
            }
            Spacer(Modifier.height(8.dp))
        }
        items(catalog.search(query)) { dev ->
            Card(modifier = Modifier.fillMaxWidth()) {
                Column(modifier = Modifier.padding(12.dp)) {
                    Text("${dev.name} (${dev.identifier})", fontWeight = FontWeight.Bold)
                    Text("${dev.soc} · ${dev.socChip} · ${dev.arch} · ${dev.tier.label}")
                    Text("Board ${dev.board} · ${dev.displayPx[0]}×${dev.displayPx[1]} · RAM ${dev.ramMb} MB")
                    if (expanded == dev.identifier) {
                        Spacer(Modifier.height(6.dp))
                        dev.firmwares.forEach { fw ->
                            Row(
                                modifier = Modifier.fillMaxWidth().padding(vertical = 4.dp),
                                horizontalArrangement = Arrangement.SpaceBetween,
                            ) {
                                Column(modifier = Modifier.weight(1f)) {
                                    Text("${fw.osName ?: "iOS"} ${fw.version ?: "—"} (${fw.build ?: "—"})")
                                    fw.ipswFilename?.let { Text(it, style = MaterialTheme.typography.bodySmall) }
                                }
                                StatusChip(fw.status)
                                if (fw.id != null) {
                                    TextButton(onClick = { onSelectFirmware(dev, fw) }) { Text("Open") }
                                }
                            }
                        }
                        TextButton(onClick = { expanded = null }) { Text("Collapse") }
                    } else {
                        TextButton(onClick = { expanded = dev.identifier }) { Text("Show firmwares") }
                    }
                }
            }
        }
    }
}

@Composable
private fun DeviceScreen(
    device: Device,
    firmware: FirmwareEntry,
    onBack: () -> Unit,
    onAddToLibrary: () -> Unit,
) {
    Column(
        modifier = Modifier.fillMaxSize().padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        Text(
            "${device.name} — ${firmware.osName ?: "iOS"} ${firmware.version ?: ""}",
            style = MaterialTheme.typography.headlineSmall,
            fontWeight = FontWeight.Bold,
        )
        StatusChip(firmware.status)
        Text("${device.identifier} · board ${device.board} · ${device.soc} (${device.socChip}) · ${device.arch}")
        Text("Display ${device.displayPx[0]}×${device.displayPx[1]} · RAM ${device.ramMb} MB · Build ${firmware.build ?: "—"}")
        firmware.ipswFilename?.let { Text("IPSW: $it") }
        if (firmware.sha256 != null) {
            Text("SHA-256: ${firmware.sha256}")
        } else {
            Text("SHA-256: not available (${firmware.sha256Note ?: "no reason recorded"})")
        }
        Card(modifier = Modifier.fillMaxWidth().height(220.dp)) {
            Column(modifier = Modifier.padding(16.dp)) {
                Text(
                    if (firmware.status.isRunnable) {
                        "Engine not integrated in v0.1.0. This firmware runs on the baseline " +
                            "engine in LightTouch today; NextTouch's engine integration is the " +
                            "next roadmap phase. No screen is faked here."
                    } else {
                        "Not booting yet. The engine has no machine for this SoC yet. v0.1.0 " +
                            "ships the picker, catalog and roadmap — no boot is claimed for " +
                            "this firmware."
                    },
                )
            }
        }
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Button(onClick = onAddToLibrary) { Text("Add to library") }
            OutlinedButton(onClick = onBack) { Text("Back") }
        }
    }
}

@Composable
private fun LibraryScreen(items: List<LibraryItem>, onRemove: (LibraryItem) -> Unit) {
    LazyColumn(
        modifier = Modifier.fillMaxSize().padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        item {
            Text("Library", style = MaterialTheme.typography.headlineSmall, fontWeight = FontWeight.Bold)
            if (items.isEmpty()) Text("No devices yet. Add one from the Device picker.")
        }
        items(items) { item ->
            Card(modifier = Modifier.fillMaxWidth()) {
                Column(modifier = Modifier.padding(12.dp)) {
                    Text(item.name, fontWeight = FontWeight.Bold)
                    Text("${item.deviceIdentifier} · iOS ${item.firmwareVersion}")
                    Text("local id ${item.id.take(16)}", style = MaterialTheme.typography.bodySmall)
                    TextButton(onClick = { onRemove(item) }) { Text("Remove") }
                }
            }
        }
    }
}

@Composable
private fun SettingsScreen(
    themeChoice: ThemeChoice,
    onThemeChange: (ThemeChoice) -> Unit,
    filesDir: File,
    onWipe: () -> Unit,
) {
    var confirmWipe by remember { mutableStateOf(false) }
    var wiped by remember { mutableStateOf(false) }
    Column(
        modifier = Modifier.fillMaxSize().padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        Text("Settings", style = MaterialTheme.typography.headlineSmall, fontWeight = FontWeight.Bold)
        Text("Theme (app chrome only — an emulated screen is never tinted):")
        ThemeChoice.entries.forEach { choice ->
            Row {
                RadioButton(selected = themeChoice == choice, onClick = { onThemeChange(choice) })
                Text(choice.label, modifier = Modifier.padding(top = 12.dp))
            }
        }
        Text("Data folder: ${filesDir.name} (app-private)")
        Text("All data is local. No telemetry, no accounts, no update checks.")
        Text("Guest networking: off (default; takes effect in the engine phase)")
        Spacer(Modifier.height(12.dp))
        Text("Danger zone", fontWeight = FontWeight.Bold)
        OutlinedButton(onClick = { confirmWipe = true }) { Text("Delete all NextTouch data…") }
        if (wiped) Text("All NextTouch data deleted.")
    }
    if (confirmWipe) {
        AlertDialog(
            onDismissRequest = { confirmWipe = false },
            title = { Text("Delete all NextTouch data?") },
            text = {
                Text(
                    "This permanently deletes the app's private data folder " +
                        "(${filesDir.name}) — settings, library and all device data.",
                )
            },
            confirmButton = {
                TextButton(onClick = {
                    onWipe()
                    confirmWipe = false
                    wiped = true
                }) { Text("Delete everything") }
            },
            dismissButton = {
                TextButton(onClick = { confirmWipe = false }) { Text("Cancel") }
            },
        )
    }
}

@Composable
private fun AboutScreen() {
    Column(
        modifier = Modifier.fillMaxSize().padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        Text("About NextTouch", style = MaterialTheme.typography.headlineSmall, fontWeight = FontWeight.Bold)
        Text("Version 0.1.0")
        Text(
            "NextTouch extends the qemu-ios emulator family (LightTouch / iOS-Emulator) " +
                "toward iOS 8, 9 and 10. v0.1.0 ships the shared catalog, picker, library and " +
                "settings on Windows, Android and iOS. The A5 machine (iPhone 4S, iOS 8.4.1) " +
                "is the first engine target; no iOS 8/9/10 firmware boots yet and the picker " +
                "labels it that way.",
        )
        Text("Privacy", fontWeight = FontWeight.Bold)
        Text(
            "No telemetry, no analytics, no crash reporting, no accounts, no third-party SDKs. " +
                "The only planned network path is downloading an IPSW from Apple, on your tap, " +
                "verified by SHA-256 before use.",
        )
        Text("Licences", fontWeight = FontWeight.Bold)
        Text("NextTouch — Copyright (c) 2026 c12n Apps. GPL-2.0-or-later.")
        Text(
            "Built on: LightTouch (GPL-2.0-or-later), mada-02/iOS-Emulator (GPL-2.0-or-later), " +
                "qemu-ios / QEMU (GPL-2.0), Jetpack Compose / Kotlin (Apache-2.0), " +
                "kotlinx.serialization (Apache-2.0). JIT approach on iOS adapted from Husk " +
                "(GPL-2.0-or-later). Full notices ship in THIRD_PARTY_NOTICES.md.",
        )
        Text(
            "iPhone, iPad and iPod touch are trademarks of Apple Inc. This project is not " +
                "affiliated with Apple and does not distribute Apple firmware.",
        )
    }
}
