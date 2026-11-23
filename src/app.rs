#[path = "diagnostics.rs"]
mod diagnostics;
#[path = "security.rs"]
mod security;
#[path = "tweaks.rs"]
mod tweaks;

use eframe::egui;
use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, Instant},
};
use sysinfo::System;

const CRIMSON: egui::Color32 = egui::Color32::from_rgb(190, 42, 58);
const MUTED: egui::Color32 = egui::Color32::from_rgb(157, 162, 174);
const INTEGRITY_INTERVAL: Duration = Duration::from_secs(60);
const METRICS_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Dashboard,
    CpuGpu,
    Windows,
    Startup,
    Services,
    Debloat,
    Cleaner,
    Network,
    Processes,
    Scanner,
    Restore,
    Logs,
    Clips,
    Desktop,
    Drivers,
    Health,
}

impl Page {
    fn label(self) -> &'static str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::CpuGpu => "CPU & GPU",
            Self::Windows => "Windows Tweaks",
            Self::Startup => "Startup",
            Self::Services => "Services",
            Self::Debloat => "Optional Apps",
            Self::Cleaner => "Cleaner",
            Self::Network => "Network",
            Self::Processes => "Processes",
            Self::Scanner => "Scanner",
            Self::Restore => "Restore",
            Self::Logs => "Activity Logs",
            Self::Clips => "Crimson Clips",
            Self::Desktop => "Crimson Desktop",
            Self::Drivers => "Driver checker",
            Self::Health => "System health",
        }
    }
}

pub struct CrimsonApp {
    page: Page,
    system: System,
    clips_dir: PathBuf,
    folder_status: String,
    executable: Option<PathBuf>,
    integrity_baseline: Option<[u8; 32]>,
    integrity_status: String,
    integrity_tampered: bool,
    last_integrity_check: Instant,
    last_metrics_refresh: Instant,
    scan_tx: Sender<String>,
    scan_rx: Receiver<String>,
    scan_running: bool,
    scan_status: String,
    optimization_tx: Sender<String>,
    optimization_rx: Receiver<String>,
    optimization_running: bool,
    optimization_status: String,
    selected_tweaks: [bool; 7],
    network_host: String,
    network_result: String,
    startup_result: String,
    cleanup_result: String,
    process_result: String,
    service_result: String,
    apps_result: String,
    graphics_result: String,
    power_result: String,
    driver_result: String,
    driver_tx: Sender<String>,
    driver_rx: Receiver<String>,
    driver_running: bool,
    health_tx: Sender<String>,
    health_rx: Receiver<String>,
    health_running: bool,
    health_result: String,
}

impl Default for CrimsonApp {
    fn default() -> Self {
        let (clips_dir, folder_status) = match videos_dir() {
            Some(videos) => {
                let directory = videos.join("Crimson Clips");
                let status = match std::fs::create_dir_all(directory.join("Clips"))
                    .and_then(|_| std::fs::create_dir_all(directory.join("Exports")))
                {
                    Ok(()) => format!("Clip folders ready: {}", directory.display()),
                    Err(error) => format!("Could not create clip folders: {error}"),
                };
                (directory, status)
            }
            None => (
                PathBuf::new(),
                "Could not determine the Videos directory; clip folders were not created."
                    .to_owned(),
            ),
        };
        let executable = std::env::current_exe().ok();
        let integrity_baseline = executable
            .as_deref()
            .and_then(|path| security::sha256_file(path).ok());
        let integrity_status = if integrity_baseline.is_some() {
            "Baseline captured. Runtime checks run every 60 seconds.".to_owned()
        } else {
            "Integrity cannot be verified: executable path or baseline is unavailable. Crimson is locked.".to_owned()
        };
        let (scan_tx, scan_rx) = mpsc::channel();
        let (optimization_tx, optimization_rx) = mpsc::channel();
        let (health_tx, health_rx) = mpsc::channel();
        let (driver_tx, driver_rx) = mpsc::channel();

        Self {
            page: Page::Dashboard,
            system: System::new(),
            clips_dir,
            folder_status,
            executable,
            integrity_baseline,
            integrity_status,
            // Fail closed if the executable cannot be fingerprinted at startup.
            integrity_tampered: integrity_baseline.is_none(),
            // Run the first verification on the first UI update, not after a 60-second delay.
            last_integrity_check: Instant::now() - INTEGRITY_INTERVAL,
            last_metrics_refresh: Instant::now() - METRICS_INTERVAL,
            scan_tx,
            scan_rx,
            scan_running: false,
            scan_status: "No Defender scan has been requested.".to_owned(),
            optimization_tx,
            optimization_rx,
            optimization_running: false,
            optimization_status: "No optimization has been applied in this session.".to_owned(),
            selected_tweaks: [false; 7],
            network_host: "example.com".to_owned(),
            network_result: "Run a DNS lookup and four-packet ping test against a hostname."
                .to_owned(),
            startup_result: "Startup inventory has not been scanned.".to_owned(),
            cleanup_result: "Temporary-file preview has not been run.".to_owned(),
            process_result: "Process inventory has not been scanned.".to_owned(),
            service_result: "Service inventory has not been scanned.".to_owned(),
            apps_result: "Installed-app inventory has not been scanned.".to_owned(),
            graphics_result: "Graphics adapter inventory has not been scanned.".to_owned(),
            power_result: "Active power plan has not been checked.".to_owned(),
            driver_result: "Driver inventory has not been scanned.".to_owned(),
            driver_tx,
            driver_rx,
            driver_running: false,
            health_tx,
            health_rx,
            health_running: false,
            health_result: "No Windows health scan has been run.".to_owned(),
        }
    }
}

fn videos_dir() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(|profile| PathBuf::from(profile).join("Videos"))
}

impl CrimsonApp {
    fn refresh_metrics(&mut self) {
        self.system.refresh_cpu_all();
        self.system.refresh_memory();
        self.last_metrics_refresh = Instant::now();
    }

    fn check_integrity(&mut self) {
        if self.last_integrity_check.elapsed() < INTEGRITY_INTERVAL {
            return;
        }
        self.last_integrity_check = Instant::now();

        let (Some(executable), Some(expected)) =
            (self.executable.as_deref(), self.integrity_baseline)
        else {
            self.integrity_tampered = true;
            self.integrity_status =
                "Integrity cannot be verified: executable baseline is missing. Crimson is locked."
                    .to_owned();
            return;
        };

        match security::sha256_file(executable) {
            Ok(actual) if actual == expected => {
                self.integrity_status =
                    "Executable unchanged since this session started.".to_owned();
            }
            Ok(_) => {
                self.integrity_tampered = true;
                self.integrity_status =
                    "WARNING: executable bytes changed after launch. Sensitive actions are locked."
                        .to_owned();
            }
            Err(error) => {
                self.integrity_tampered = true;
                self.integrity_status =
                    format!("Integrity verification failed: {error}. Crimson is locked.");
            }
        }
    }

    fn start_defender_scan(&mut self) {
        if self.scan_running || self.integrity_tampered {
            return;
        }
        let Some(executable) = self.executable.clone() else {
            self.scan_status = "Cannot locate the running executable.".to_owned();
            return;
        };

        let sender = self.scan_tx.clone();
        self.scan_running = true;
        self.scan_status = "Microsoft Defender scan is running…".to_owned();
        thread::spawn(move || {
            let result = security::run_defender_scan(&executable);
            let _ = sender.send(result);
        });
    }

    fn collect_scan_result(&mut self) {
        if let Ok(result) = self.scan_rx.try_recv() {
            self.scan_running = false;
            self.scan_status = result;
        }
    }

    fn spawn_optimization(
        &mut self,
        task: impl FnOnce() -> Result<String, String> + Send + 'static,
    ) {
        if self.optimization_running {
            return;
        }
        self.optimization_running = true;
        self.optimization_status = "Preparing a restore point and settings snapshot…".to_owned();
        let sender = self.optimization_tx.clone();
        thread::spawn(move || {
            let status = match task() {
                Ok(message) => message,
                Err(error) => format!("Optimization cancelled or failed: {error}"),
            };
            let _ = sender.send(status);
        });
    }

    fn start_selected_tweaks(&mut self) {
        let ids: Vec<tweaks::TweakId> = tweaks::TweakId::ALL
            .into_iter()
            .zip(self.selected_tweaks)
            .filter_map(|(id, selected)| selected.then_some(id))
            .collect();
        self.spawn_optimization(move || tweaks::apply_selected(&ids));
    }

    fn undo_latest_optimization(&mut self) {
        self.spawn_optimization(tweaks::undo_latest);
    }

    fn collect_optimization_result(&mut self) {
        if let Ok(result) = self.optimization_rx.try_recv() {
            self.optimization_running = false;
            self.optimization_status = result;
        }
    }

    fn start_driver_scan(&mut self) {
        if self.driver_running {
            return;
        }
        self.driver_running = true;
        self.driver_result = "Reading installed driver inventory…".to_owned();
        let sender = self.driver_tx.clone();
        thread::spawn(move || {
            let result = diagnostics::driver_inventory()
                .unwrap_or_else(|error| format!("Driver inventory failed: {error}"));
            let _ = sender.send(result);
        });
    }

    fn collect_driver_result(&mut self) {
        if let Ok(result) = self.driver_rx.try_recv() {
            self.driver_running = false;
            self.driver_result = result;
        }
    }

    fn start_health_scan(&mut self) {
        if self.health_running {
            return;
        }
        self.health_running = true;
        self.health_result = "Running DISM component scan and SFC verification…".to_owned();
        let sender = self.health_tx.clone();
        thread::spawn(move || {
            let result = diagnostics::system_health_scan()
                .unwrap_or_else(|error| format!("Health scan failed: {error}"));
            let _ = sender.send(result);
        });
    }

    fn collect_health_result(&mut self) {
        if let Ok(result) = self.health_rx.try_recv() {
            self.health_running = false;
            self.health_result = result;
        }
    }

    fn health_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("Windows component health");
        ui.label("Checks the Windows component store and protected system files. These are scan-only operations; Crimson will not repair or replace files without a separate confirmed action.");
        ui.add_space(10.0);
        ui.add_enabled_ui(!self.health_running, |ui| {
            if ui.button("Run system health scan").clicked() {
                self.start_health_scan();
            }
        });
        if self.health_running {
            ui.spinner();
            ui.label("This can take several minutes. Keep Crimson open.");
        }
        ui.add_space(8.0);
        ui.label(egui::RichText::new(&self.health_result).monospace());
        ui.label("If corruption is detected, review the output and back up important files before attempting Windows repair.");
    }

    fn drivers_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("Driver inventory");
        ui.label("Lists installed Plug-and-Play driver versions and dates. A date alone cannot prove a driver is outdated; compare against your laptop maker or component manufacturer's support page.");
        ui.add_enabled_ui(!self.driver_running, |ui| {
            if ui.button("Scan installed drivers").clicked() {
                self.start_driver_scan();
            }
        });
        if self.driver_running {
            ui.spinner();
            ui.label("Collecting device and driver metadata…");
        }
        ui.add_space(8.0);
        ui.label(egui::RichText::new(&self.driver_result).monospace());
        ui.separator();
        ui.label("Safe update path: Windows Update → Advanced options → Optional updates → Driver updates, or the Samsung / Intel support page for the exact device model.");
        ui.label("Crimson does not bulk-download or force-install drivers. Incorrect drivers can disable Wi-Fi, graphics, audio, or startup.");
    }

    fn windows_tweaks_page(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("Tweak library");
                ui.label(egui::RichText::new("Choose exactly which Windows settings to change. Every setting appears once.").color(MUTED));
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let count = self.selected_tweaks.iter().filter(|selected| **selected).count();
                ui.label(egui::RichText::new(format!("{count} selected")).strong().color(CRIMSON));
            });
        });
        ui.add_space(14.0);
        ui.columns(3, |columns| {
            columns[0].vertical(|ui| {
                tweak_category_header(ui, "GAMING", "Game capture and Windows game scheduling");
                tweak_card(
                    ui,
                    &mut self.selected_tweaks,
                    tweaks::TweakId::GameCaptureOff,
                );
                tweak_card(ui, &mut self.selected_tweaks, tweaks::TweakId::GameModeOn);
            });
            columns[1].vertical(|ui| {
                tweak_category_header(
                    ui,
                    "PERFORMANCE",
                    "Windows visual effects and responsiveness",
                );
                tweak_card(
                    ui,
                    &mut self.selected_tweaks,
                    tweaks::TweakId::TransparencyOff,
                );
                tweak_card(
                    ui,
                    &mut self.selected_tweaks,
                    tweaks::TweakId::VisualEffectsPerformance,
                );
            });
            columns[2].vertical(|ui| {
                tweak_category_header(
                    ui,
                    "PRIVACY",
                    "Optional suggestions and notification content",
                );
                tweak_card(
                    ui,
                    &mut self.selected_tweaks,
                    tweaks::TweakId::DisableAppSuggestions,
                );
                tweak_card(
                    ui,
                    &mut self.selected_tweaks,
                    tweaks::TweakId::DisableWindowsTips,
                );
                tweak_card(
                    ui,
                    &mut self.selected_tweaks,
                    tweaks::TweakId::DisableConsumerFeatures,
                );
            });
        });
        ui.add_space(12.0);
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            let mut apply = false;
            let mut clear = false;
            ui.add_enabled_ui(!self.optimization_running, |ui| {
                apply = ui
                    .button(egui::RichText::new("Apply selected tweaks").strong())
                    .clicked();
                clear = ui.button("Clear selection").clicked();
            });
            if apply {
                self.start_selected_tweaks();
            }
            if clear {
                self.selected_tweaks = [false; 7];
            }
            if ui.button("Undo last batch").clicked() && !self.optimization_running {
                self.undo_latest_optimization();
            }
            if self.optimization_running {
                ui.spinner();
                ui.label("Applying and verifying selected settings…");
            }
        });
        ui.add_space(8.0);
        ui.label(egui::RichText::new(&self.optimization_status).color(MUTED));
        ui.label("Before changes, Crimson requests a Windows restore point and saves the exact original registry values. If verification fails, it attempts rollback. No Defender, Firewall, Windows Update, services, drivers, or installed apps are changed.");
    }

    fn startup_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("Startup inventory");
        ui.label("Read-only inventory of common Windows Run and RunOnce registry locations. Crimson does not disable or delete startup entries.");
        if ui.button("Scan startup entries").clicked() {
            self.startup_result = diagnostics::startup_inventory()
                .unwrap_or_else(|error| format!("Inventory failed: {error}"));
        }
        ui.add_space(8.0);
        ui.label(egui::RichText::new(&self.startup_result).monospace());
    }

    fn cleaner_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("Temporary-file cleanup preview");
        ui.label("This first pass measures top-level files in the current user's TEMP folder. It does not delete anything.");
        if ui.button("Scan temporary files").clicked() {
            self.cleanup_result = diagnostics::cleanup_preview()
                .unwrap_or_else(|error| format!("Preview failed: {error}"));
        }
        ui.add_space(8.0);
        ui.label(egui::RichText::new(&self.cleanup_result).monospace());
    }

    fn network_diagnostics_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("DNS and latency diagnostics");
        ui.label("Enter a hostname to perform a bounded DNS lookup and four-packet ping test. This does not change DNS servers, adapters or firewall rules.");
        ui.horizontal(|ui| {
            ui.label("Host");
            ui.text_edit_singleline(&mut self.network_host);
            if ui.button("Run test").clicked() {
                self.network_result = diagnostics::network_test(&self.network_host)
                    .unwrap_or_else(|error| format!("Test failed: {error}"));
            }
        });
        ui.add_space(8.0);
        ui.label(egui::RichText::new(&self.network_result).monospace());
        ui.add_space(16.0);
        ui.separator();
        self.network_page(ui);
    }

    fn processes_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("Running processes");
        ui.label("Read-only process inventory from Windows tasklist. Crimson does not terminate processes or change their priority.");
        if ui.button("Refresh process list").clicked() {
            self.process_result = diagnostics::process_inventory()
                .unwrap_or_else(|error| format!("Process inventory failed: {error}"));
        }
        ui.add_space(8.0);
        ui.label(egui::RichText::new(&self.process_result).monospace());
    }

    fn services_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("Windows services");
        ui.label("Read-only inventory of Windows services and their current states. No service is stopped, disabled, or set to manual.");
        if ui.button("Scan Windows services").clicked() {
            self.service_result = diagnostics::service_inventory()
                .unwrap_or_else(|error| format!("Service inventory failed: {error}"));
        }
        ui.add_space(8.0);
        ui.label(egui::RichText::new(&self.service_result).monospace());
        ui.label("Service optimization is intentionally not automatic: disabling the wrong service can break networking, updates, audio, security, or boot.");
    }

    fn debloat_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("Installed applications");
        ui.label("Read-only inventory of registered desktop applications. This page does not uninstall apps, remove Store packages, or edit Windows components.");
        if ui.button("Scan installed applications").clicked() {
            self.apps_result = diagnostics::installed_apps_inventory()
                .unwrap_or_else(|error| format!("App inventory failed: {error}"));
        }
        ui.add_space(8.0);
        ui.label(egui::RichText::new(&self.apps_result).monospace());
        ui.label("Crimson will not label software as malware based only on its name or publisher. Use Microsoft Defender for security scanning.");
    }

    fn restore_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("Restore and rollback");
        ui.label("Undo the most recent Crimson optimization batch using its saved original registry values.");
        ui.add_enabled_ui(!self.optimization_running, |ui| {
            if ui.button("Undo latest Crimson optimization").clicked() {
                self.undo_latest_optimization();
            }
        });
        if self.optimization_running {
            ui.spinner();
        }
        ui.label(egui::RichText::new(&self.optimization_status).color(MUTED));
        ui.separator();
        ui.label("Windows System Restore is separate from Crimson rollback. Open System Protection in Windows to review or restore a Windows restore point.");
    }

    fn logs_page(&self, ui: &mut egui::Ui) {
        ui.heading("Activity log");
        ui.label("Current-session status. This build does not claim to maintain a tamper-proof audit log.");
        ui.group(|ui| {
            ui.label("Executable integrity");
            ui.label(&self.integrity_status);
            ui.separator();
            ui.label("Defender scan");
            ui.label(&self.scan_status);
            ui.separator();
            ui.label("Optimization / rollback");
            ui.label(&self.optimization_status);
            ui.separator();
            ui.label("Startup inventory");
            ui.label(&self.startup_result);
            ui.separator();
            ui.label("Temporary-file preview");
            ui.label(&self.cleanup_result);
            ui.separator();
            ui.label("Network diagnostics");
            ui.label(&self.network_result);
        });
    }

    fn sidebar(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new("CRIMSON").color(CRIMSON).strong());
                ui.label(egui::RichText::new("SYSTEM BOOST").small().color(MUTED));
            });
            ui.add_space(16.0);
            let sections: &[(Page, &str)] = &[
                (Page::Dashboard, "CORE"),
                (Page::CpuGpu, "CPU & GPU"),
                (Page::Windows, "Tweaks"),
                (Page::Startup, "Startup"),
                (Page::Services, "Services"),
                (Page::Debloat, "Optional Apps"),
                (Page::Cleaner, "Cleaner"),
                (Page::Network, "Network"),
                (Page::Processes, "Processes"),
                (Page::Scanner, "Scanner"),
                (Page::Restore, "Restore"),
                (Page::Logs, "Activity Logs"),
                (Page::Clips, "Crimson Clips"),
                (Page::Desktop, "Crimson Desktop"),
                (Page::Drivers, "Driver checker"),
                (Page::Health, "System health"),
            ];
            for (page, label) in sections {
                if *label == "CORE" {
                    ui.label(egui::RichText::new(*label).small().color(MUTED));
                    continue;
                }
                let selected = self.page == *page;
                let text = if selected {
                    egui::RichText::new(*label)
                        .strong()
                        .color(egui::Color32::WHITE)
                } else {
                    egui::RichText::new(*label).color(MUTED)
                };
                if ui.selectable_label(selected, text).clicked() {
                    self.page = *page;
                }
            }
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                ui.label(
                    egui::RichText::new("NIGHT OPS  •  v0.1.0")
                        .small()
                        .color(MUTED),
                );
            });
        });
    }

    fn dashboard(&self, ui: &mut egui::Ui) {
        let cpus = self.system.cpus();
        let cpu = if cpus.is_empty() {
            0.0
        } else {
            cpus.iter().map(|core| core.cpu_usage()).sum::<f32>() / cpus.len() as f32
        };
        let memory_total = self.system.total_memory();
        let memory_used = self.system.used_memory();
        let memory_percent = if memory_total > 0 {
            memory_used as f32 / memory_total as f32 * 100.0
        } else {
            0.0
        };

        ui.heading("System overview");
        ui.label(
            egui::RichText::new("Live readings from this PC. No fabricated FPS or GPU values.")
                .color(MUTED),
        );
        ui.add_space(14.0);
        egui::Grid::new("metrics")
            .num_columns(3)
            .spacing([12.0, 12.0])
            .show(ui, |ui| {
                metric(ui, "CPU usage", format!("{cpu:.1}%"));
                metric(ui, "Memory used", format!("{memory_percent:.1}%"));
                metric(
                    ui,
                    "Memory",
                    format!(
                        "{:.1} / {:.1} GB",
                        memory_used as f64 / 1_073_741_824.0,
                        memory_total as f64 / 1_073_741_824.0
                    ),
                );
                ui.end_row();
            });
        ui.add_space(20.0);
        ui.group(|ui| {
            ui.label(egui::RichText::new("Safety policy").strong().color(CRIMSON));
            ui.label("Crimson includes individually selectable reversible tweaks plus read-only startup, cleanup and network diagnostics. Destructive cleanup and service changes remain disabled until individually verified.");
            ui.label("Microsoft Defender and Windows Firewall are never disabled or reconfigured by this build.");
        });
        ui.add_space(12.0);
        ui.label("Available now: individually selectable, reversible Windows tweaks, startup inventory, temporary-file preview, and DNS/ping diagnostics.");
    }

    fn scanner_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("Security checks");
        ui.label("Crimson checks whether its running executable changes after launch. This is a runtime tamper alarm, not proof that the downloaded file was authentic before launch.");
        ui.add_space(10.0);
        ui.group(|ui| {
            ui.label(
                egui::RichText::new("Executable integrity")
                    .strong()
                    .color(CRIMSON),
            );
            let color = if self.integrity_tampered {
                egui::Color32::RED
            } else {
                MUTED
            };
            ui.label(egui::RichText::new(&self.integrity_status).color(color));
            ui.label("The SHA-256 baseline is captured at startup and checked once per minute.");
        });
        ui.add_space(10.0);
        ui.group(|ui| {
            ui.label(egui::RichText::new("Microsoft Defender Antivirus").strong());
            ui.label("Runs a custom scan of the current Crimson executable using Microsoft's MpCmdRun utility. Scan results are reported by Defender; Crimson does not replace its detection engine.");
            ui.add_enabled_ui(!self.scan_running && !self.integrity_tampered, |ui| {
                if ui.button("Scan Crimson executable with Defender").clicked() {
                    self.start_defender_scan();
                }
            });
            ui.label(&self.scan_status);
            ui.label("For detections and remediation details, open Windows Security → Virus & threat protection → Protection history.");
        });
        if self.integrity_tampered {
            ui.add_space(10.0);
            ui.colored_label(egui::Color32::RED, "Crimson has locked its scan action because its executable changed during this session. Close the app and reinstall from a trusted release. No files will be automatically deleted.");
        }
    }

    fn cpu_gpu_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("Hardware compatibility");
        let cpu = self
            .system
            .cpus()
            .first()
            .map(|processor| processor.brand().trim())
            .filter(|brand| !brand.is_empty())
            .unwrap_or("CPU information unavailable");
        ui.label(format!("Detected CPU: {cpu}"));
        ui.add_space(8.0);
        ui.group(|ui| {
            ui.label(egui::RichText::new("Resizable BAR (ReBAR)").strong().color(CRIMSON));
            ui.label("ReBAR is a PCIe firmware and hardware capability, not a Windows registry tweak.");
            ui.label("It requires a compatible CPU/platform, motherboard firmware, and supported graphics device. The BIOS normally needs UEFI boot, Above 4G Decoding, and Re-Size BAR enabled.");
            ui.label("Crimson will not force-enable ReBAR or write undocumented firmware settings. Unsupported hardware cannot gain ReBAR through software.");
            ui.label("Status: Not verified by this app. Check the motherboard/laptop manufacturer specifications and BIOS. If the firmware has no Re-Size BAR option, treat the device as unsupported.");
        });
        ui.add_space(8.0);
        ui.group(|ui| {
            ui.label(
                egui::RichText::new("Detected graphics adapter and driver")
                    .strong()
                    .color(CRIMSON),
            );
            if ui.button("Inspect graphics adapter").clicked() {
                self.graphics_result = diagnostics::graphics_inventory()
                    .unwrap_or_else(|error| format!("Graphics inspection failed: {error}"));
            }
            ui.label(egui::RichText::new(&self.graphics_result).monospace());
        });
        ui.add_space(8.0);
        ui.group(|ui| {
            ui.label(egui::RichText::new("Active Windows power plan").strong().color(CRIMSON));
            if ui.button("Check active power plan").clicked() {
                self.power_result = diagnostics::power_plan()
                    .unwrap_or_else(|error| format!("Power-plan check failed: {error}"));
            }
            ui.label(egui::RichText::new(&self.power_result).monospace());
            ui.label("Crimson reports the plan only. It does not force High Performance or disable thermal and power management.");
        });
        ui.add_space(8.0);
        ui.group(|ui| {
            ui.label(egui::RichText::new("XMP / EXPO").strong().color(CRIMSON));
            ui.label("Memory profiles are configured in BIOS/UEFI and depend on the memory controller, motherboard and RAM. Crimson does not change memory timings or voltage.");
        });
        ui.add_space(8.0);
        ui.group(|ui| {
            ui.label(egui::RichText::new("CPU parking and power plans").strong().color(CRIMSON));
            ui.label("Windows already manages core scheduling and parking dynamically. Forcing every core online can increase heat and power use without improving frame times. Crimson does not apply blanket un-parking or overclocking.");
        });
    }

    fn network_page(&self, ui: &mut egui::Ui) {
        ui.heading("Windows network protection");
        ui.label(
            "Crimson does not add, remove, or modify Windows Defender Firewall rules or profiles.",
        );
        ui.label("This app does not open a listening network port, host a server, or require inbound connections.");
        ui.label("Microsoft Defender Antivirus scans files. Windows Defender Firewall filters network traffic. They are separate Windows security components, and Crimson leaves the firewall configuration alone.");
        ui.add_space(10.0);
        ui.label("If a future feature genuinely needs network access, it must request a narrowly scoped rule with clear user consent. No broad allow-all rule will be created.");
    }

    fn clips_page(&self, ui: &mut egui::Ui) {
        ui.heading("Crimson Clips");
        ui.label("Instant replay frontend — recording integration is not active in this build.");
        ui.add_space(12.0);
        ui.group(|ui| {
            ui.label("Clip library");
            ui.monospace(self.clips_dir.join("Clips").display().to_string());
            ui.label("Exports");
            ui.monospace(self.clips_dir.join("Exports").display().to_string());
            ui.separator();
            ui.label(&self.folder_status);
        });
        ui.add_space(8.0);
        ui.label("The app will only show CAPTURING after it can verify that a recorder is actually active.");
    }

    fn desktop_page(&self, ui: &mut egui::Ui) {
        ui.heading("Crimson Desktop");
        ui.label("Windows taskbar and wallpaper customization is not enabled in this scaffold.");
        ui.add_space(8.0);
        ui.label("• Windows 10 and Windows 11 require separate compatibility paths.");
        ui.label(
            "• No Explorer replacement or unsupported shell patch will be applied automatically.",
        );
        ui.label(
            "• A restore option and original-layout snapshot are required before customization.",
        );
    }
}

fn tweak_category_header(ui: &mut egui::Ui, title: &str, subtitle: &str) {
    ui.group(|ui| {
        ui.set_min_height(72.0);
        ui.label(
            egui::RichText::new(title)
                .strong()
                .color(CRIMSON)
                .size(15.0),
        );
        ui.label(egui::RichText::new(subtitle).small().color(MUTED));
    });
    ui.add_space(8.0);
}

fn tweak_card(ui: &mut egui::Ui, selected: &mut [bool; 7], id: tweaks::TweakId) {
    ui.group(|ui| {
        ui.set_min_height(116.0);
        ui.checkbox(
            &mut selected[id.index()],
            egui::RichText::new(id.label()).strong(),
        );
        ui.add_space(4.0);
        ui.label(egui::RichText::new(id.description()).small().color(MUTED));
    });
    ui.add_space(8.0);
}

fn metric(ui: &mut egui::Ui, title: &str, value: String) {
    ui.group(|ui| {
        ui.set_min_width(190.0);
        ui.label(egui::RichText::new(title).small().color(MUTED));
        ui.heading(egui::RichText::new(value).color(egui::Color32::WHITE));
    });
}

impl eframe::App for CrimsonApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.last_metrics_refresh.elapsed() >= METRICS_INTERVAL {
            self.refresh_metrics();
        }
        self.check_integrity();
        self.collect_scan_result();
        self.collect_optimization_result();
        self.collect_health_result();
        self.collect_driver_result();
        ctx.request_repaint_after(Duration::from_secs(1));

        let mut style = (*ctx.style()).clone();
        style.visuals = egui::Visuals::dark();
        style.visuals.panel_fill = egui::Color32::from_rgb(12, 13, 18);
        style.visuals.window_fill = egui::Color32::from_rgb(18, 19, 26);
        style.visuals.extreme_bg_color = egui::Color32::from_rgb(8, 9, 13);
        style.visuals.faint_bg_color = egui::Color32::from_rgb(26, 27, 36);
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(24, 25, 33);
        style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(31, 32, 42);
        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(55, 29, 39);
        style.visuals.widgets.active.bg_fill = CRIMSON;
        style.visuals.selection.bg_fill = CRIMSON;
        style.spacing.item_spacing = egui::vec2(10.0, 9.0);
        style.spacing.button_padding = egui::vec2(12.0, 8.0);
        ctx.set_style(style);

        if self.integrity_tampered {
            egui::CentralPanel::default()
                .frame(egui::Frame::default().fill(egui::Color32::from_rgb(8, 8, 10)))
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(70.0);
                        ui.heading(egui::RichText::new("☠").size(112.0).color(egui::Color32::from_rgb(220, 35, 48)));
                        ui.add_space(12.0);
                        ui.heading(egui::RichText::new("CRIMSON INTEGRITY FAILURE").strong().color(egui::Color32::from_rgb(220, 35, 48)));
                        ui.add_space(12.0);
                        ui.label(egui::RichText::new(&self.integrity_status).color(egui::Color32::WHITE));
                        ui.label(egui::RichText::new("Crimson has locked its controls. No Windows settings, firewall rules, or other applications were changed.").color(MUTED));
                        ui.add_space(18.0);
                        ui.label(egui::RichText::new("Close Crimson and reinstall from a trusted, verified release.").strong().color(egui::Color32::WHITE));
                        ui.label(egui::RichText::new("Automatic file repair is unavailable until a signed installer and independent recovery helper are configured.").small().color(MUTED));
                    });
                });
            return;
        }

        egui::TopBottomPanel::top("crimson_header")
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("CRIMSON SYSTEM BOOST").strong().color(CRIMSON).size(17.0));
                        ui.label(egui::RichText::new(format!("{}  /  Windows utility", self.page.label())).small().color(MUTED));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new("LOCAL • REVERSIBLE • NO TELEMETRY").small().color(MUTED));
                    });
                });
                ui.add_space(4.0);
            });

        egui::SidePanel::left("navigation")
            .resizable(false)
            .exact_width(205.0)
            .show(ctx, |ui| self.sidebar(ui));
        egui::TopBottomPanel::bottom("performance_status_bar")
            .resizable(false)
            .show(ctx, |ui| {
                let cpus = self.system.cpus();
                let cpu = if cpus.is_empty() {
                    0.0
                } else {
                    cpus.iter().map(|core| core.cpu_usage()).sum::<f32>() / cpus.len() as f32
                };
                let total_memory = self.system.total_memory();
                let used_memory = self.system.used_memory();
                let memory_percent = if total_memory > 0 {
                    used_memory as f32 / total_memory as f32 * 100.0
                } else {
                    0.0
                };
                ui.horizontal_wrapped(|ui| {
                    ui.colored_label(CRIMSON, "CRIMSON LIVE");
                    ui.separator();
                    ui.label(format!("CPU {cpu:.0}%"));
                    ui.separator();
                    ui.label(format!("RAM {memory_percent:.0}%"));
                    ui.separator();
                    ui.label("GPU — telemetry unavailable");
                    ui.separator();
                    ui.label("GAME FPS — no frame-capture source connected");
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(8.0);
            ui.heading(self.page.label());
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| match self.page {
                Page::Dashboard => self.dashboard(ui),
                Page::Scanner => self.scanner_page(ui),
                Page::Windows => self.windows_tweaks_page(ui),
                Page::CpuGpu => self.cpu_gpu_page(ui),
                Page::Startup => self.startup_page(ui),
                Page::Cleaner => self.cleaner_page(ui),
                Page::Network => self.network_diagnostics_page(ui),
                Page::Processes => self.processes_page(ui),
                Page::Services => self.services_page(ui),
                Page::Debloat => self.debloat_page(ui),
                Page::Restore => self.restore_page(ui),
                Page::Logs => self.logs_page(ui),
                Page::Clips => self.clips_page(ui),
                Page::Desktop => self.desktop_page(ui),
                Page::Drivers => self.drivers_page(ui),
                Page::Health => self.health_page(ui),
            });
        });
    }
}
