//! 500-item Windows optimization review catalog.
//! Entries are unique subsystem × review-dimension pairs. They are guided checks,
//! not 500 automatic registry modifications or guaranteed FPS boosts.

#[derive(Clone, Copy)]
pub struct CatalogEntry {
    pub id: usize,
    pub category: &'static str,
    pub subsystem: &'static str,
    pub review: &'static str,
    pub guidance: &'static str,
}

const SUBSYSTEMS: [(&str, &str); 50] = [
    ("Gaming", "Xbox Game Bar"),
    ("Gaming", "Game Mode"),
    ("Gaming", "DirectX runtime"),
    ("Gaming", "Graphics scheduling"),
    ("Gaming", "Fullscreen presentation"),
    ("Gaming", "Variable refresh rate"),
    ("Gaming", "VSync configuration"),
    ("Gaming", "Frame pacing"),
    ("Gaming", "Game launchers"),
    ("Gaming", "Roblox client"),
    ("Graphics", "Intel graphics adapter"),
    ("Graphics", "Display resolution"),
    ("Graphics", "Display refresh rate"),
    ("Graphics", "HDR configuration"),
    ("Graphics", "GPU driver package"),
    ("Graphics", "Graphics control panel"),
    ("Graphics", "Hardware acceleration"),
    ("Graphics", "Shader cache"),
    ("Graphics", "Multi-monitor layout"),
    ("Graphics", "Windowed game optimizations"),
    ("Windows", "Windows Update"),
    ("Windows", "Optional driver updates"),
    ("Windows", "System component store"),
    ("Windows", "System file integrity"),
    ("Windows", "Restore point protection"),
    ("Windows", "Startup applications"),
    ("Windows", "Scheduled tasks"),
    ("Windows", "Background applications"),
    ("Windows", "Notification settings"),
    ("Windows", "Visual effects"),
    ("Storage", "System drive capacity"),
    ("Storage", "Temporary files"),
    ("Storage", "SSD health"),
    ("Storage", "TRIM status"),
    ("Storage", "Drive optimization schedule"),
    ("Storage", "Storage Sense"),
    ("Storage", "Windows Search index"),
    ("Storage", "Delivery Optimization cache"),
    ("Storage", "Recycle Bin"),
    ("Storage", "Crash dump files"),
    ("Network", "Wi-Fi adapter"),
    ("Network", "Wi-Fi signal quality"),
    ("Network", "DNS resolver"),
    ("Network", "Network driver"),
    ("Network", "TCP/IP configuration"),
    ("Network", "Proxy configuration"),
    ("Network", "Metered connection"),
    ("Network", "Firewall profile"),
    ("Network", "Network latency"),
    ("Network", "Roblox connectivity"),
];

const REVIEW_DIMENSIONS: [(&str, &str); 10] = [
    ("Current state", "Read the current setting and compare it with the Windows or vendor default."),
    ("Compatibility", "Check whether this feature is supported by the detected Windows version and hardware."),
    ("Driver dependency", "Identify the driver or vendor package that controls this feature; do not install third-party driver bundles."),
    ("Startup impact", "Review whether related software launches at sign-in and whether the user actually needs it."),
    ("Resource impact", "Measure or estimate CPU, memory, disk, or GPU impact before recommending a change."),
    ("Gaming relevance", "Explain whether this setting can affect frame pacing, latency, loading, or only visual appearance."),
    ("Update status", "Check the official Windows Update or hardware-vendor path for a supported update."),
    ("Security boundary", "Verify that improving performance does not require weakening Defender, Firewall, updates, or account protection."),
    ("Recovery path", "Record the supported way to restore the previous setting before recommending a change."),
    ("User validation", "Give the user a repeatable before-and-after test and explain what result would justify keeping a change."),
];

pub const CATALOG_COUNT: usize = SUBSYSTEMS.len() * REVIEW_DIMENSIONS.len();

pub fn entries() -> Vec<CatalogEntry> {
    let mut entries = Vec::with_capacity(CATALOG_COUNT);
    for (subsystem_index, (category, subsystem)) in SUBSYSTEMS.iter().enumerate() {
        for (review_index, (review, guidance)) in REVIEW_DIMENSIONS.iter().enumerate() {
            entries.push(CatalogEntry {
                id: subsystem_index * REVIEW_DIMENSIONS.len() + review_index + 1,
                category,
                subsystem,
                review,
                guidance,
            });
        }
    }
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_contains_500_unique_entries() {
        let entries = entries();
        assert_eq!(entries.len(), 500);
        let mut ids = entries.iter().map(|entry| entry.id).collect::<Vec<_>>();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 500);
        assert_eq!(entries.first().unwrap().id, 1);
        assert_eq!(entries.last().unwrap().id, 500);
    }
}
