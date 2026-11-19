use eframe::egui;
use std::path::PathBuf;
use sysinfo::System;

const CRIMSON: egui::Color32 = egui::Color32::from_rgb(190, 42, 58);
const PANEL: egui::Color32 = egui::Color32::from_rgb(29, 31, 38);
const MUTED: egui::Color32 = egui::Color32::from_rgb(157, 162, 174);

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
}

impl Default for CrimsonApp {
    fn default() -> Self {
        let clips_dir = videos_dir().join("Crimson Clips");
        let folder_status = match std::fs::create_dir_all(clips_dir.join("Clips"))
            .and_then(|_| std::fs::create_dir_all(clips_dir.join("Exports")))
        {
            Ok(()) => format!("Clip folders ready: {}", clips_dir.display()),
            Err(error) => format!("Could not create clip folders: {error}"),
        };

        Self {
            page: Page::Dashboard,
            system: System::new_all(),
            clips_dir,
            folder_status,
        }
    }
}

fn videos_dir() -> PathBuf {
    if let Ok(profile) = std::env::var("USERPROFILE") {
        PathBuf::from(profile).join("Videos")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join("Videos")
    } else {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    }
}

impl CrimsonApp {
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
                    egui::RichText::new(*label).strong().color(egui::Color32::WHITE)
                } else {
                    egui::RichText::new(*label).color(MUTED)
                };
                if ui.selectable_label(selected, text).clicked() {
                    self.page = *page;
                }
            }
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                ui.label(egui::RichText::new("NIGHT OPS  •  v0.1.0").small().color(MUTED));
            });
        });
    }

    fn dashboard(&mut self, ui: &mut egui::Ui) {
        self.system.refresh_all();
        let cpu = self.system.global_cpu_info().cpu_usage();
        let memory_total = self.system.total_memory();
        let memory_used = self.system.used_memory();
        let memory_percent = if memory_total > 0 {
            memory_used as f32 / memory_total as f32 * 100.0
        } else {
            0.0
        };

        ui.heading("System overview");
        ui.label(egui::RichText::new("Live readings from this PC. No fabricated FPS or GPU values.").color(MUTED));
        ui.add_space(14.0);

        egui::Grid::new("metrics").num_columns(3).spacing([12.0, 12.0]).show(ui, |ui| {
            metric(ui, "CPU usage", format!("{cpu:.1}%"));
            metric(ui, "Memory used", format!("{memory_percent:.1}%"));
            metric(ui, "Memory", format!("{:.1} / {:.1} GB", memory_used as f64 / 1_073_741_824.0, memory_total as f64 / 1_073_741_824.0));
            ui.end_row();
        });

        ui.add_space(20.0);
        ui.group(|ui| {
            ui.label(egui::RichText::new("Safety policy").strong().color(CRIMSON));
            ui.label("Every system tweak must create and verify a restore point before changing settings.");
            ui.label("If restore-point creation or verification fails, the requested change is cancelled.");
            ui.label("Defender, Windows Firewall, Windows Update and essential Windows services are protected.");
        });

        ui.add_space(12.0);
        ui.label("This first scaffold is monitoring-only. Optimization controls will stay unavailable until each action has compatibility checks, snapshots and a tested undo path.");
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
        ui.label("Planned integration: Medal-supported replay hotkey or a separately implemented capture backend.");
        ui.label("The app will only show CAPTURING after it can verify that a recorder is actually active.");
    }

    fn desktop_page(&self, ui: &mut egui::Ui) {
        ui.heading("Crimson Desktop");
        ui.label("Windows taskbar and wallpaper customization is not enabled in this scaffold.");
        ui.add_space(8.0);
        ui.bullet_text("Windows 10 and Windows 11 require separate compatibility paths.");
        ui.bullet_text("Centered pinned apps, left-side Start, hidden search and taskbar transparency need version-aware testing.");
        ui.bullet_text("No Explorer replacement or unsupported shell patch will be applied automatically.");
        ui.bullet_text("A restore option and original-layout snapshot are required before any customization.");
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
        let mut style = (*ctx.style()).clone();
        style.visuals = egui::Visuals::dark();
        style.visuals.panel_fill = PANEL;
        style.visuals.window_fill = PANEL;
        style.visuals.widgets.active.bg_fill = CRIMSON;
        style.visuals.selection.bg_fill = CRIMSON;
        ctx.set_style(style);

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
                Page::Clips => self.clips_page(ui),
                Page::Desktop => self.desktop_page(ui),
                page => {
                    ui.label(format!("{} module is planned but not implemented yet.", page.label()));
                    ui.label("No system settings have been changed.");
                }
            });
        });
    }
}
