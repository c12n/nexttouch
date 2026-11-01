// NextTouch desktop — Copyright (c) 2026 c12n Apps
// SPDX-License-Identifier: GPL-2.0-or-later
//
//! NextTouch v0.1.0 desktop shell (egui/eframe) for Windows + Linux.
//!
//! What v0.1.0 ships: the shared device/firmware picker, the library, the
//! settings (three-state theme, delete-all-data) and the About/Legal
//! screens, all over the Rust catalog + core crates. The emulator engine
//! (qemu-ios for the legacy range, the new A5/A6 machines for iOS 8/9/10)
//! is NOT integrated yet — the device screen says so plainly and every
//! iOS 8/9/10 firmware is labelled "Not booting yet". No fake screens,
//! no fake boot claims.
//!
//! Privacy: this binary contains no networking code at all in v0.1.0.
//! The only future network path is the IPSW downloader (Apple hosts only,
//! explicit user tap) from the project's privacy audit.
//! Themes: Follow system by default; the theme applies to the app chrome
//! only — an emulated framebuffer (engine phase) is never tinted.

#![windows_subsystem = "windows"]

use eframe::egui;
use nexttouch_catalog::{Catalog, Device, FirmwareEntry, FirmwareStatus};
use nexttouch_core::library::Library;
use nexttouch_core::settings::{Settings, ThemePreference};
use std::path::PathBuf;

fn data_dir() -> PathBuf {
    // App-private storage. Windows: %LOCALAPPDATA%\NextTouch
    // Linux: $XDG_DATA_HOME/NextTouch or ~/.local/share/NextTouch
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        if !local.is_empty() {
            return PathBuf::from(local).join("NextTouch");
        }
    }
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        if !xdg.is_empty() {
            return PathBuf::from(xdg).join("NextTouch");
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".local").join("share").join("NextTouch");
    }
    PathBuf::from("NextTouch-data")
}

fn now_utc_string() -> String {
    // No chrono dependency: seconds-since-epoch is enough for a created-at
    // stamp in v0.1.0 (displayed nowhere; ordering only).
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix:{secs}")
}

#[derive(Clone, Copy, PartialEq)]
enum Page {
    Picker,
    Library,
    DeviceScreen,
    Settings,
    About,
}

struct NextTouchApp {
    catalog: Catalog,
    settings: Settings,
    library: Library,
    data_dir: PathBuf,
    page: Page,
    search: String,
    selected_device: Option<String>,
    selected_firmware: Option<String>,
    confirm_wipe: bool,
    status_line: String,
}

impl NextTouchApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let dir = data_dir();
        let settings = Settings::load(&dir);
        apply_theme(&cc.egui_ctx, settings.theme);
        NextTouchApp {
            catalog: Catalog::embedded(),
            library: Library::load(&dir),
            settings,
            data_dir: dir,
            page: Page::Picker,
            search: String::new(),
            selected_device: None,
            selected_firmware: None,
            confirm_wipe: false,
            status_line: String::new(),
        }
    }

    fn selected(&self) -> Option<(&Device, &FirmwareEntry)> {
        let ident = self.selected_device.as_deref()?;
        let dev = self.catalog.device(ident)?;
        let fw_id = self.selected_firmware.as_deref()?;
        dev.firmwares
            .iter()
            .find(|f| f.id.as_deref() == Some(fw_id))
            .map(|f| (dev, f))
    }

    fn status_chip(ui: &mut egui::Ui, status: FirmwareStatus) {
        let (fill, text) = match status {
            FirmwareStatus::Tested => (egui::Color32::from_rgb(30, 110, 50), egui::Color32::WHITE),
            FirmwareStatus::TestedSlow => (egui::Color32::from_rgb(140, 110, 20), egui::Color32::WHITE),
            FirmwareStatus::Experimental => (egui::Color32::from_rgb(60, 80, 140), egui::Color32::WHITE),
            FirmwareStatus::NotBootingYet => (egui::Color32::from_rgb(120, 45, 45), egui::Color32::WHITE),
        };
        egui::Frame::none()
            .fill(fill)
            .rounding(egui::Rounding::same(4.0))
            .inner_margin(egui::Margin::symmetric(6.0, 2.0))
            .show(ui, |ui| {
                ui.label(egui::RichText::new(status.label()).color(text).small());
            });
    }

    fn picker_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("Device picker");
        ui.label(
            "Pick a device, then a firmware. Legacy firmwares (iPhone OS 1 – iOS 7) carry the \
             baseline engine's real status. Every iOS 8 / 9 / 10 firmware is labelled \
             \"Not booting yet\": the A5/A6 engine work is in development and v0.1.0 does not \
             boot them. Nothing here overstates what the engine can do.",
        );
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut self.search);
        });
        ui.add_space(4.0);

        ui.label(egui::RichText::new("Quick start (development defaults)").strong());
        let quick: Vec<(String, String, String)> = self
            .catalog
            .quick_start_entries()
            .iter()
            .map(|(d, f)| {
                (
                    d.identifier.clone(),
                    f.id.clone().unwrap_or_default(),
                    format!("{} — iOS {}", d.name, f.version.clone().unwrap_or_default()),
                )
            })
            .collect();
        ui.horizontal_wrapped(|ui| {
            for (ident, fw_id, label) in &quick {
                if ui.button(label).clicked() {
                    self.selected_device = Some(ident.clone());
                    self.selected_firmware = Some(fw_id.clone());
                    self.page = Page::DeviceScreen;
                }
            }
        });
        ui.add_space(8.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            let devices: Vec<&Device> = self.catalog.search_devices(&self.search);
            for dev in devices {
                let is_selected = self.selected_device.as_deref() == Some(dev.identifier.as_str());
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        let title = format!("{}  ({})", dev.name, dev.identifier);
                        if ui
                            .selectable_label(is_selected, egui::RichText::new(title).strong())
                            .clicked()
                        {
                            self.selected_device = Some(dev.identifier.clone());
                            self.selected_firmware = None;
                        }
                        ui.label(format!("{} · {} · {}", dev.soc, dev.soc_chip, dev.arch));
                        ui.label(dev.tier.label());
                    });
                    if is_selected {
                        ui.label(format!(
                            "Board {} · Display {}×{} · RAM {} MB",
                            dev.board, dev.display_px[0], dev.display_px[1], dev.ram_mb
                        ));
                        for fw in &dev.firmwares {
                            ui.horizontal(|ui| {
                                Self::status_chip(ui, fw.status);
                                let label = format!(
                                    "{} {} ({})",
                                    fw.os_name.clone().unwrap_or_default(),
                                    fw.version.clone().unwrap_or_else(|| "—".into()),
                                    fw.build.clone().unwrap_or_else(|| "—".into())
                                );
                                if ui.button(label).clicked() {
                                    self.selected_firmware = fw.id.clone();
                                    if self.selected_firmware.is_some() {
                                        self.page = Page::DeviceScreen;
                                    }
                                }
                                if let Some(file) = &fw.ipsw_filename {
                                    ui.label(egui::RichText::new(file).small().weak());
                                }
                            });
                        }
                    }
                });
                ui.add_space(4.0);
            }
        });
    }

    fn device_screen_page(&mut self, ui: &mut egui::Ui) {
        let Some((dev, fw)) = self.selected() else {
            ui.label("No device selected. Pick one in the Device picker.");
            return;
        };
        // Clone the display data so buttons below can borrow self mutably.
        let (name, ident, board, soc, chip, arch, disp, ram) = (
            dev.name.clone(),
            dev.identifier.clone(),
            dev.board.clone(),
            dev.soc.clone(),
            dev.soc_chip.clone(),
            dev.arch.clone(),
            dev.display_px,
            dev.ram_mb,
        );
        let (version, build, os_name, status, filename, sha, note, fw_id) = (
            fw.version.clone().unwrap_or_default(),
            fw.build.clone().unwrap_or_default(),
            fw.os_name.clone().unwrap_or_default(),
            fw.status,
            fw.ipsw_filename.clone(),
            fw.sha256.clone(),
            fw.sha256_note.clone(),
            fw.id.clone().unwrap_or_default(),
        );

        ui.heading(format!("{name} — {os_name} {version}"));
        ui.horizontal(|ui| {
            Self::status_chip(ui, status);
            ui.label(format!("{ident} · board {board} · {soc} ({chip}) · {arch}"));
        });
        ui.label(format!("Display {}×{} · RAM {} MB · Build {}", disp[0], disp[1], ram, build));
        if let Some(file) = &filename {
            ui.label(format!("IPSW: {file}"));
        }
        match &sha {
            Some(h) => ui.label(format!("SHA-256: {h}")),
            None => ui.label(format!(
                "SHA-256: not available{}",
                note.map(|n| format!(" ({n})")).unwrap_or_default()
            )),
        };
        ui.add_space(8.0);

        // Guest framebuffer area — a placeholder that tells the truth.
        // (Engine phase: a raw texture with NO tint is rendered here.)
        let (rect, _) = ui.allocate_exact_size(egui::vec2(320.0, 480.0), egui::Sense::hover());
        ui.painter().rect_filled(rect, 6.0, egui::Color32::from_gray(24));
        ui.painter().rect_stroke(
            rect,
            6.0,
            egui::Stroke::new(1.0, egui::Color32::from_gray(128)),
        );
        let msg = if status.is_runnable() {
            "Engine not integrated in v0.1.0.\nThis firmware runs on the baseline engine\nin LightTouch today; NextTouch's engine\nintegration is the next roadmap phase."
        } else {
            "Not booting yet.\nThe engine has no machine for this SoC yet.\nv0.1.0 ships the picker, catalog and roadmap —\nno boot is claimed for this firmware."
        };
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            msg,
            egui::FontId::proportional(13.0),
            egui::Color32::from_gray(200),
        );
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            if ui.button("Add to library").clicked() {
                let id = self.library.add_device(
                    &format!("{name} ({version})"),
                    &ident,
                    &fw_id,
                    &version,
                    &now_utc_string(),
                );
                let _ = self.library.save(&self.data_dir);
                self.status_line = format!("Added to library ({})", &id[..8]);
                self.page = Page::Library;
            }
            if ui.button("Back to picker").clicked() {
                self.page = Page::Picker;
            }
        });
        if !self.status_line.is_empty() {
            ui.label(&self.status_line);
        }
    }

    fn library_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("Library");
        if self.library.devices.is_empty() {
            ui.label("No devices yet. Add one from the Device picker.");
            return;
        }
        let mut remove: Option<String> = None;
        for dev in &self.library.devices {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(&dev.name).strong());
                    ui.label(format!(
                        "{} · {} {}",
                        dev.device_identifier, "iOS", dev.firmware_version
                    ));
                    if dev.dirty {
                        ui.label(egui::RichText::new("dirty — snapshot restore offered on launch").color(egui::Color32::from_rgb(200, 120, 40)));
                    }
                    if ui.button("Remove").clicked() {
                        remove = Some(dev.id.clone());
                    }
                });
                ui.label(
                    egui::RichText::new(format!("local id {}", &dev.id[..16]))
                        .small()
                        .weak(),
                );
            });
            ui.add_space(4.0);
        }
        if let Some(id) = remove {
            let _ = self.library.remove_device(&id);
            let _ = self.library.save(&self.data_dir);
        }
    }

    fn settings_page(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("Settings");
        ui.label("Theme (applies to the app chrome only — an emulated screen is never tinted):");
        let mut theme = self.settings.theme;
        for t in ThemePreference::all() {
            if ui.radio_value(&mut theme, t, t.label()).changed() {
                self.settings.theme = theme;
                apply_theme(ctx, theme);
                let _ = self.settings.save(&self.data_dir);
            }
        }
        ui.add_space(8.0);
        ui.label(format!("Data folder: {}", self.data_dir.display()));
        ui.label("All data is app-private and local. No telemetry, no accounts, no update checks.");
        ui.label(format!(
            "Guest networking: {} (off by default; takes effect in the engine phase)",
            if self.settings.guest_network_enabled { "on" } else { "off" }
        ));
        ui.add_space(12.0);
        ui.label(egui::RichText::new("Danger zone").strong());
        if !self.confirm_wipe {
            if ui.button("Delete all NextTouch data…").clicked() {
                self.confirm_wipe = true;
            }
        } else {
            ui.label(format!(
                "This permanently deletes {} — settings, library and all device data.",
                self.data_dir.display()
            ));
            ui.horizontal(|ui| {
                if ui.button("Yes, delete everything").clicked() {
                    let _ = nexttouch_core::privacy::delete_all_data(&self.data_dir);
                    self.library = Library::default();
                    self.settings = Settings::default();
                    self.confirm_wipe = false;
                    self.status_line = "All NextTouch data deleted.".to_string();
                }
                if ui.button("Cancel").clicked() {
                    self.confirm_wipe = false;
                }
            });
        }
        if !self.status_line.is_empty() {
            ui.label(&self.status_line);
        }
    }

    fn about_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("About NextTouch");
        ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
        ui.label(
            "NextTouch extends the qemu-ios emulator family (LightTouch / iOS-Emulator) \
             toward iOS 8, 9 and 10. v0.1.0 ships the shared catalog, picker, library and \
             settings on Windows, Android and iOS. The A5 machine (iPhone 4S, iOS 8.4.1) is \
             the first engine target; no iOS 8/9/10 firmware boots yet and the picker \
             labels it that way.",
        );
        ui.add_space(6.0);
        ui.label(egui::RichText::new("Privacy").strong());
        ui.label(
            "No telemetry, no analytics, no crash reporting, no accounts, no third-party \
             SDKs. The only planned network path is downloading an IPSW from Apple, on your \
             tap, verified by SHA-256 before use.",
        );
        ui.add_space(6.0);
        ui.label(egui::RichText::new("Licences").strong());
        ui.label("NextTouch — Copyright (c) 2026 c12n Apps. GPL-2.0-or-later.");
        ui.label(
            "Built on: LightTouch (GPL-2.0-or-later), mada-02/iOS-Emulator (GPL-2.0-or-later), \
             qemu-ios / QEMU (GPL-2.0), egui/eframe (MIT/Apache-2.0), serde (MIT/Apache-2.0). \
             JIT approach on iOS adapted from Husk (GPL-2.0-or-later). \
             Full notices ship in THIRD_PARTY_NOTICES.md with every release.",
        );
        ui.label("iPhone, iPad and iPod touch are trademarks of Apple Inc. This project is not affiliated with Apple and does not distribute Apple firmware.");
    }
}

fn apply_theme(ctx: &egui::Context, theme: ThemePreference) {
    // DeepSeek audit: persist the preference, never a resolved theme;
    // fallback is Dark; chrome only.
    let mut options = egui::Options::default();
    options.theme_preference = match theme {
        ThemePreference::System => egui::ThemePreference::System,
        ThemePreference::Light => egui::ThemePreference::Light,
        ThemePreference::Dark => egui::ThemePreference::Dark,
    };
    options.fallback_theme = egui::Theme::Dark;
    ctx.options_mut(|o| *o = options);
}

impl eframe::App for NextTouchApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("nav").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("NextTouch");
                ui.separator();
                if ui
                    .selectable_label(self.page == Page::Picker, "Device picker")
                    .clicked()
                {
                    self.page = Page::Picker;
                }
                if ui
                    .selectable_label(self.page == Page::Library, "Library")
                    .clicked()
                {
                    self.page = Page::Library;
                }
                if ui
                    .selectable_label(self.page == Page::Settings, "Settings")
                    .clicked()
                {
                    self.page = Page::Settings;
                }
                if ui
                    .selectable_label(self.page == Page::About, "About")
                    .clicked()
                {
                    self.page = Page::About;
                }
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| match self.page {
            Page::Picker => self.picker_page(ui),
            Page::DeviceScreen => self.device_screen_page(ui),
            Page::Library => self.library_page(ui),
            Page::Settings => self.settings_page(ui, ctx),
            Page::About => self.about_page(ui),
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([980.0, 720.0])
            .with_title("NextTouch"),
        ..Default::default()
    };
    eframe::run_native(
        "NextTouch",
        options,
        Box::new(|cc| Ok(Box::new(NextTouchApp::new(cc)))),
    )
}
