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
- OneClick Optimizer profile for disabling Xbox Game Bar background recording, enabling Windows Game Mode, and disabling transparency effects.
- Platinum+ adds Windows visual-effects performance mode.
- Windows Tweaks includes reversible current-user controls for consumer-experience suggestions, Start menu app suggestions, and Windows tips.
- CPU & GPU page explains ReBAR and XMP/EXPO hardware/firmware requirements. It does not pretend a Windows app can enable BIOS-only features or force unsupported hardware to support them.
- WinUtil-inspired Windows Tweaks page with individual selections, batch apply, status reporting, and undo of the latest Crimson batch.
- Tweaks are restricted to seven allow-listed per-user registry settings. Crimson does not disable Defender, Firewall, Windows Update, drivers, or Windows services, and does not remove installed apps.
- These are independent implementations inspired by publicly documented optimizer workflows; Crimson does not copy third-party source code.
- Before each batch, Crimson requests UAC approval to create a Windows System Restore point, saves the original values, verifies each write, and attempts automatic rollback if a write fails. If Windows cannot confirm the restore point, the batch is cancelled.
- CPU and memory readings through "sysinfo", refreshed once per second rather than on every UI frame.
- SHA-256 runtime integrity check of the running executable, checked once per minute against a baseline captured at launch.
- Full-screen red/black lockout warning when the running executable changes after launch; this is detection only, not automatic repair.
- Optional Microsoft Defender custom scan of the running executable through Microsoft's MpCmdRun utility.
- No Windows Defender Firewall rules or profiles are changed, and the app does not open inbound network ports.
- Creates "%USERPROFILE%\Videos\Crimson Clips\Clips" and "Exports" on first launch when a user profile path is available.
- Crimson Clips? Not implemented yet. No actual recording.
- Crimson Desktop? Also not implemented. No taskbar modifications.
- Startup, Services, Optional Apps, Cleaner, and several other navigation pages remain planned modules. They are not presented as working features yet.

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
