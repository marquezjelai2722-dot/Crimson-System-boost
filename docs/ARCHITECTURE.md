# Architecture

## Current implementation

The current executable is a native Rust desktop application built with eframe/egui. System CPU and memory readings are collected with sysinfo. The UI currently lives in `src/app.rs` and the application entry point is `src/main.rs`.

The current branch is a scaffold, not a completed optimizer. Most navigation pages are explicit placeholders. Crimson Clips creates its local folder structure on first launch, but does not record video. Crimson Desktop does not modify the Windows taskbar.

## Planned boundaries

As functionality grows, keep responsibilities separated:

- **ui** — presentation and user interaction only;
- **system_info** — read-only hardware and OS inventory;
- **safety** — compatibility gates and protected-component policy;
- **restore** — restore-point creation and verification;
- **snapshots** — capture and compare original settings;
- **tweaks** — individually testable, reversible operations;
- **audit** — append-only local activity records;
- **clips** — capture integration and clip-library management;
- **desktop** — version-aware taskbar settings and rollback.

The UI must call typed application services. Avoid embedding registry edits, service control, or shell commands directly in UI callbacks.

## Privilege model

Run without elevation by default. Request administrator permission only for a specific operation that demonstrably requires it. Explain the requested permission before elevation. Never keep an elevated helper running unnecessarily.

## Data locations

User recordings and exports belong under the user's Videos directory, not inside the installation directory. Uninstallers must preserve user recordings unless the user explicitly chooses otherwise.
