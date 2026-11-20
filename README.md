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

Early development. Not publicly tested. Expect unfinished features.

This isn't some magical one-click FPS booster yet. It's an early desktop scaffold, and I'd rather be honest about that than pretend everything works.

# What's actually implemented:

- Rust + egui/eframe desktop dashboard with Night Ops styling.
- CPU and memory readings through "sysinfo", refreshed once per second rather than on every UI frame.
- SHA-256 runtime integrity check of the running executable, checked once per minute against a baseline captured at launch.
- Optional Microsoft Defender custom scan of the running executable through Microsoft's MpCmdRun utility.
- No Windows Defender Firewall rules or profiles are changed, and the app does not open inbound network ports.
- Creates "%USERPROFILE%\Videos\Crimson Clips\Clips" and "Exports" on first launch when a user profile path is available.
- Crimson Clips? Not implemented yet. No actual recording.
- Crimson Desktop? Also not implemented. No taskbar modifications.
- The other optimization pages? Mostly placeholders. They don't modify Windows settings.

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
3. Reversible optimizations with proper audit logs.
4. Crimson Clips — recording integration, tray support, visible recording status, and configurable replay length.
5. Crimson Desktop — Windows 10/11 taskbar customization with restoration support.
6. Installer — automatic folder creation without deleting someone's recordings during uninstall.

No imaginary features marked as completed. No pretending a placeholder is a working feature.

# Scope

Windows 10 and Windows 11 are the target operating systems.

Individual features will have their own compatibility requirements. Windows 11 support does not magically make unsupported hardware compatible.

Anyway, that's Crimson System Boost.

It's unfinished. It's probably going to have bugs. There is a lot of work left, and I am fully aware of that.

If you find something broken, report it. If you know how to fix it, even better.

Now excuse me. I have another Rust error to stare at and a coffee that has probably stopped being coffee.
