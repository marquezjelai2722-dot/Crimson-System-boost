# Crimson System Boost

A Windows-first, Rust-based gaming diagnostics and reversible optimization project.
(I strictly advised to update to windows 10-11 or this won't work. And this pure rust. Just email me if you want to contribute to this project, Make sure you have a repo built using python, or rust and knows how to actually code. But i do advised to know how to use AI to help you or ease your work. But don't be a lazy ass who only knows how to vibecode. Email: MarquezJelai2722@gmail.com)

## Current status

This repository contains an early native desktop scaffold, not a finished optimizer or installer.

- Rust + egui/eframe desktop dashboard with Night Ops styling.
- Live CPU and memory readings through sysinfo.
- Creates `%USERPROFILE%\\Videos\\Crimson Clips\\Clips` and `Exports` on first launch.
- Crimson Clips and Crimson Desktop are clearly marked as not implemented; no recording or taskbar changes occur.
- Other optimization pages are placeholders and do not change system settings.

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
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo run
```

Windows release build:

```powershell
cargo build --release
```

The executable is written to `target\\release\\crimson-system-boost.exe`. Build output is ignored by Git.

## Repository files

- `.gitignore` — excludes Rust build output, local secrets, logs, temporary files, editor state and installer artifacts.
- `.editorconfig`, `.gitattributes`, `rustfmt.toml` — consistent formatting and line endings.
- `SECURITY.md` — vulnerability reporting and security invariants.
- `CONTRIBUTING.md` — development and pull-request rules.
- `docs/ARCHITECTURE.md` — planned module boundaries and privilege model.
- `docs/SAFETY_MODEL.md` — mandatory preflight, protected components and rollback policy.
- `docs/BUILDING.md` — Windows build instructions.

## Roadmap

1. Monitoring and hardware/OS compatibility detection.
2. Verified restore-point service and settings snapshots.
3. Reversible per-setting optimizations with audit logs.
4. Crimson Clips: validated capture integration, tray residency, visible capture state and configurable replay length.
5. Crimson Desktop: separately tested Windows 10/11 taskbar customization with restore support.
6. Installer that creates the clip folders and preserves recordings on uninstall.

## Scope

Windows 10/11 are the target operating systems, but individual features will declare their supported versions. Windows 11 support is not implied for unsupported hardware.
