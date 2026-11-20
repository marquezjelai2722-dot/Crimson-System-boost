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
const PANEL: egui::Color32 = egui::Color32::from_rgb(29, 31, 38);
const MUTED: egui::Color32 = egui::Color32::from_rgb(157, 162, 174);
const INTEGRITY_INTERVAL: Duration = Duration::from_secs(60);
const METRICS_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Dashboard,
    Gaming,
    CpuGpu,
    Windows,
    Startup,
    Services,
    Debloat,
    Cleaner,
    Network,
    Scanner,
    Restore,
    Logs,
    Clips,
    Desktop,
}

impl Page {
    fn label(self) -> &'static str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::Gaming => "Gaming Boost",
            Self::CpuGpu => "CPU & GPU",
            Self::Windows => "Windows Tweaks",
            Self::Startup => "Startup",
            Self::Services => "Services",
            Self::Debloat => "Optional Apps",
            Self::Cleaner => "Cleaner",
            Self::Network => "Network",
            Self::Scanner => "Scanner",
            Self::Restore => "Restore",
            Self::Logs => "Activity Logs",
            Self::Clips => "Crimson Clips",
            Self::Desktop => "Crimson Desktop",
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
    selected_tweaks: [bool; 4],
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
            selected_tweaks: [true, true, true, false],
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

    fn start_preset(&mut self, preset: tweaks::Preset) {
        self.spawn_optimization(move || tweaks::apply_preset(preset));
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

    fn gaming_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("Gaming performance profiles");
        ui.label("Real Windows settings. No fake FPS counter, timer-resolution forcing, core-parking hacks, or security disabling.");
        ui.add_space(12.0);

        ui.group(|ui| {
            ui.label(
                egui::RichText::new("OneClick Optimizer")
                    .strong()
                    .color(CRIMSON),
            );
            ui.label("Balanced gaming profile:");
            ui.label("• Disable Xbox Game Bar background recording and automatic captures.");
            ui.label("• Enable Windows Game Mode.");
            ui.label("• Disable Windows transparency effects.");
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new("Platinum+ Optimizer")
                    .strong()
                    .color(CRIMSON),
            );
            ui.label(
                "Includes the OneClick profile, plus Windows visual-effects performance mode.",
            );
            ui.label(
                "This can reduce visual polish. It does not guarantee a specific FPS increase.",
            );
        });

        let enabled = !self.optimization_running;
        let mut apply_one_click = false;
        let mut apply_platinum = false;
        ui.add_enabled_ui(enabled, |ui| {
            ui.horizontal_wrapped(|ui| {
                apply_one_click = ui.button("Apply OneClick Optimizer").clicked();
                apply_platinum = ui.button("Apply Platinum+ Optimizer").clicked();
            });
        });
        if apply_one_click {
            self.start_preset(tweaks::Preset::OneClick);
        } else if apply_platinum {
            self.start_preset(tweaks::Preset::PlatinumPlus);
        }

        ui.add_space(10.0);
        if self.optimization_running {
            ui.spinner();
        }
        ui.label(egui::RichText::new(&self.optimization_status).color(MUTED));
        ui.label("Applying a profile requests UAC approval to create a Windows System Restore point. If Windows cannot confirm the restore point, Crimson cancels the changes.");
    }

    fn windows_tweaks_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("Windows Utility — Tweaks");
        ui.label("Select individual changes, apply them as one reviewed batch, and undo the latest Crimson batch.");
        ui.add_space(10.0);

        ui.group(|ui| {
            ui.label(egui::RichText::new("Gaming").strong().color(CRIMSON));
            ui.checkbox(
                &mut self.selected_tweaks[0],
                tweaks::TweakId::GameCaptureOff.label(),
            );
            ui.label(tweaks::TweakId::GameCaptureOff.description());
            ui.checkbox(
                &mut self.selected_tweaks[1],
                tweaks::TweakId::GameModeOn.label(),
            );
            ui.label(tweaks::TweakId::GameModeOn.description());
        });
        ui.add_space(8.0);
        ui.group(|ui| {
            ui.label(
                egui::RichText::new("Appearance and responsiveness")
                    .strong()
                    .color(CRIMSON),
            );
            ui.checkbox(
                &mut self.selected_tweaks[2],
                tweaks::TweakId::TransparencyOff.label(),
            );
            ui.label(tweaks::TweakId::TransparencyOff.description());
            ui.checkbox(
                &mut self.selected_tweaks[3],
                tweaks::TweakId::VisualEffectsPerformance.label(),
            );
            ui.label(tweaks::TweakId::VisualEffectsPerformance.description());
        });

        let enabled = !self.optimization_running;
        let mut apply = false;
        let mut undo = false;
        ui.add_enabled_ui(enabled, |ui| {
            ui.horizontal_wrapped(|ui| {
                apply = ui.button("Apply selected tweaks").clicked();
                undo = ui.button("Undo latest Crimson batch").clicked();
            });
        });
        if apply {
            self.start_selected_tweaks();
        } else if undo {
            self.undo_latest_optimization();
        }

        ui.add_space(10.0);
        if self.optimization_running {
            ui.spinner();
        }
        ui.label(egui::RichText::new(&self.optimization_status).color(MUTED));
        ui.label("Each batch is limited to the four listed HKCU DWORD settings. No services, drivers, Defender, Firewall, Windows Update, or installed apps are modified.");
        ui.label("A restore point and exact original registry values are saved before changes. Undo restores missing values by deleting only the values Crimson created.");
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
                (Page::Gaming, "Gaming Boost"),
                (Page::CpuGpu, "CPU & GPU"),
                (Page::Windows, "Windows Tweaks"),
                (Page::Startup, "Startup"),
                (Page::Services, "Services"),
                (Page::Debloat, "Optional Apps"),
                (Page::Cleaner, "Cleaner"),
                (Page::Network, "Network"),
                (Page::Scanner, "Scanner"),
                (Page::Restore, "Restore"),
                (Page::Logs, "Activity Logs"),
                (Page::Clips, "Crimson Clips"),
                (Page::Desktop, "Crimson Desktop"),
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
            ui.label("System-changing features remain disabled until compatibility checks, snapshots and tested rollback are implemented.");
            ui.label("Microsoft Defender and Windows Firewall are never disabled or reconfigured by this build.");
        });
        ui.add_space(12.0);
        ui.label("This build is monitoring-only. Optimization controls stay unavailable until each action has compatibility checks, snapshots and a tested undo path.");
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
        ctx.request_repaint_after(Duration::from_secs(1));

        let mut style = (*ctx.style()).clone();
        style.visuals = egui::Visuals::dark();
        style.visuals.panel_fill = PANEL;
        style.visuals.window_fill = PANEL;
        style.visuals.widgets.active.bg_fill = CRIMSON;
        style.visuals.selection.bg_fill = CRIMSON;
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

        egui::SidePanel::left("navigation")
            .resizable(false)
            .exact_width(205.0)
            .show(ctx, |ui| self.sidebar(ui));
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(8.0);
            ui.heading(self.page.label());
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| match self.page {
                Page::Dashboard => self.dashboard(ui),
                Page::Scanner => self.scanner_page(ui),
                Page::Gaming => self.gaming_page(ui),
                Page::Windows => self.windows_tweaks_page(ui),
                Page::Network => self.network_page(ui),
                Page::Clips => self.clips_page(ui),
                Page::Desktop => self.desktop_page(ui),
                page => {
                    ui.label(format!(
                        "{} module is planned but not implemented yet.",
                        page.label()
                    ));
                    ui.label("No system settings have been changed.");
                }
            });
        });
    }
}
