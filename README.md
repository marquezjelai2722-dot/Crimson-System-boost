# Crimson System Boost

A Windows-first gaming diagnostics and optimization project. Written in Rust because apparently I wasn't suffering enough already.

So, what is this?

Basically, I'm trying to build a Windows utility that monitors your system and helps improve gaming performance without turning your operating system into a crime scene.

No fake FPS boosts. No random registry tweaks from some 2014 YouTube tutorial. No disabling Windows Defender because someone said it gives +300 FPS.

Just actual diagnostics, useful optimizations, and settings you can undo when something inevitably goes wrong.

Still working on it. Still debugging. Still drinking coffee. Probably going to find another bug five minutes after writing this.

Before you contribute

This project targets Windows 10 and Windows 11. Older Windows versions aren't supported. Please update if you're still running something ancient.

Want to help? Email me.

I'm looking for people who actually know what they're doing with Rust. You don't need to be some legendary software engineer, but you should at least understand the code you're submitting.

AI is fine. Use it. Let it help you debug, understand unfamiliar code, or save yourself some time. I use AI too. There's nothing wrong with that.

But please, don't generate 800 lines of code, understand absolutely none of it, and throw it into a pull request. If you can't explain what your code does, you probably shouldn't be submitting it yet.

Learn. Test. Debug. Break things responsibly. Fix them afterward.

Contact: MarquezJelai2722@gmail.com

# Current status

Early development. Expect unfinished features. This branch is still undergoing Windows CI and runtime verification.

This isn't a magical one-click FPS booster. It applies a small allow-listed set of Windows settings and provides diagnostics; it does not promise a fixed FPS gain.

# What's actually implemented:

- Rust + egui/eframe desktop dashboard with Night Ops styling.
- A categorized tweak library with individually selectable, reversible per-user registry settings.
- Tweak library with separate Gaming, Performance, and Privacy columns; each tweak appears once and can be selected independently. Batch apply uses original-value snapshots, restore-point request, write verification, rollback on failure, and undo of the latest Crimson batch.
- Live CPU and memory readings through sysinfo.
- Fixed bottom performance status bar with live CPU and RAM readings; game FPS and GPU telemetry are explicitly marked unavailable until a real capture/telemetry backend is connected.
- SHA-256 runtime integrity check and lockout warning; this is runtime tamper detection, not proof of download authenticity or automatic repair.
- Optional Microsoft Defender custom scan of the running Crimson executable.
- Read-only startup inventory for common Run and RunOnce registry locations.
- Read-only process and Windows service inventories; no process termination, priority changes, or service control.
- Read-only inventory of registered desktop applications; no app removal or Store package debloating.
- Temporary-folder size preview; no file deletion.
- Bounded DNS and four-packet ping diagnostics; no DNS, adapter, or Firewall settings are changed.
- Graphics adapter/driver and active Windows power-plan inspection; no overclocking, core-unparking, or power-plan forcing.
- Restore page can undo the latest Crimson registry-tweak batch. Activity page shows current-session status, not a tamper-proof persistent audit log.
- ReBAR and XMP/EXPO are hardware/firmware features. Crimson does not fake-enable unsupported capabilities.
- No Defender, Firewall, Windows Update, drivers, or essential Windows services are disabled.
- Creates "%USERPROFILE%\\Videos\\Crimson Clips\\Clips" and "Exports" on first launch when a user profile path is available.
- Crimson Clips has no recording backend yet. Crimson Desktop does not modify the taskbar or wallpaper.
- Lossless Scaling-style graphics scaling and frame generation are not implemented; they require a separate graphics capture/rendering backend.

The diagnostics pages inspect state only. Startup controls, destructive cleanup, service changes, app removal, process controls, and persistent audit logging remain future work until they have safe rollback and compatibility checks.

Yes, there's still a lot to do. I know.

Safety contract

I'm not spending all this time building an optimizer just for it to destroy someone's Windows installation.

So here's the deal:

- Windows Defender, Windows Firewall, Windows Update, drivers, Microsoft Store dependencies, and essential services are protected. The current build does not modify firewall rules or profiles.
- Every future tweak must check compatibility, create and verify a Windows restore point, save the original setting, log the change, and provide a tested undo option.
- If a restore point can't be created and verified, the associated change gets cancelled. No shortcuts.
- Built-in app removal must be optional and explain what it removes and any compatibility risks.
- Restore points don't back up personal files. Make actual backups.
- No blind registry tweaks, random core-parking changes, forced timer resolution, or disabling security features for performance.

I want better performance, not a Windows installation that needs an exorcism.

Build locally

You'll need the stable Rust toolchain.

Then run:

cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo run

Want the Windows release executable?

cargo build --release

It should appear here:

target\release\crimson-system-boost.exe

Build output is ignored by Git. Because nobody needs a repository full of compiled junk.

Repository files

- ".gitignore" — keeps build output, secrets, logs, temporary files, editor state, and installer artifacts out of Git.
- ".editorconfig", ".gitattributes", "rustfmt.toml" — keeps formatting and line endings consistent.
- "SECURITY.md" — security requirements and vulnerability reporting.
- "CONTRIBUTING.md" — contribution rules. Please read it before opening a PR.
- "docs/ARCHITECTURE.md" — planned module structure and privilege model.
- "docs/SAFETY_MODEL.md" — compatibility checks, protected components, and rollback requirements.
- "docs/BUILDING.md" — Windows build instructions.

# Roadmap

The stuff I actually want to finish, assuming Windows, Rust, and my remaining brain cells cooperate:

1. System monitoring and hardware/OS compatibility detection.
2. Restore-point creation and settings snapshots that are actually verified.
3. Add verified, opt-in startup controls with backups and restoration.
4. Add preview-first cleanup with explicit confirmation and conservative file eligibility rules.
5. Add safe per-game profiles and measurable before/after frame-time diagnostics.
6. Evaluate a dedicated graphics capture/scaling backend before promising Lossless Scaling-style features.
7. Add persistent, integrity-protected audit logs.
8. Crimson Clips — recording integration, tray support, visible recording status, and configurable replay length.
9. Crimson Desktop — Windows 10/11 taskbar customization with restoration support.

No imaginary features marked as completed. No pretending a placeholder is a working feature.

# Scope

Windows 10 and Windows 11 are the target operating systems.

Individual features will have their own compatibility requirements. Windows 11 support does not magically make unsupported hardware compatible.

Anyway, that's Crimson System Boost.

It's unfinished. It's probably going to have bugs. There is a lot of work left, and I am fully aware of that.

If you find something broken, report it. If you know how to fix it, even better.

Now excuse me. I have another Rust error to stare at and a coffee that has probably stopped being coffee.
