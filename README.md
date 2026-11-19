# Crimson System Boost

A Windows-first, Rust-based gaming diagnostics and reversible optimization project.

## Current scaffold

- Native desktop dashboard using Rust + egui/eframe.
- Live CPU and memory readings through sysinfo.
- Creates `%USERPROFILE%\\Videos\\Crimson Clips\\Clips` and `Exports` on first launch.
- Night Ops dark interface with Crimson navigation.
- Explicit placeholders for Crimson Clips and Crimson Desktop; neither claims to record or modify the Windows shell yet.

## Safety contract

- Windows Defender, Windows Firewall, Windows Update, drivers, Store dependencies and essential services are protected.
- Every future tweak must check compatibility, create and verify a Windows restore point, snapshot the original setting, log the action and provide a tested undo.
- If a restore point cannot be created and verified, the associated change must be cancelled.
- Optional built-in app removal is opt-in and must show purpose and compatibility warnings.
- Restore points do not back up personal files. They are not a substitute for file backups.
- No blind registry tweaks, core-parking changes, timer-resolution forcing, or security-feature disabling.

## Build locally

Install the stable Rust toolchain, then run:

```powershell
cargo fmt --check
cargo check
cargo test
cargo run
```

Windows release build:

```powershell
cargo build --release
```

The executable is written to `target\\release\\crimson-system-boost.exe`.

## Roadmap

1. Monitoring and hardware/OS compatibility detection.
2. Verified restore-point service and settings snapshots.
3. Reversible per-setting optimizations with audit logs.
4. Crimson Clips: validated Medal integration or a dedicated capture backend, tray residency, visible capture state and configurable replay length.
5. Crimson Desktop: separately tested Windows 10/11 taskbar customization with restore support.
6. Installer that creates the clip folders and preserves recordings on uninstall.

## Scope

Windows 10/11 are the target operating systems, but individual features will declare their supported versions. Windows 11 support is not implied for unsupported hardware.
