# Crimson System Boost

Android device optimization companion. The repository includes a native Android app and a private HTML UI simulation in `preview/index.html`.

## Build
Push to `main` or run **Actions → Build Android APK → Run workflow**. GitHub Actions builds a debug APK and uploads it as the `crimson-system-boost-debug` artifact.

## Safety model
- Device inventory is informational; package names alone never justify removal.
- No automatic deletion of Android system packages or disabling of core services.
- Gaming firewall rules require explicit VPN consent and should only control network access.
- File scanning is limited to files the user selects or grants access to.
- Quarantine isolates a copy in app-private storage; destructive deletion requires confirmation.
- Shizuku is optional and advanced actions must be permission-gated, individually explained, and reversible where possible.
- Never disable Android thermal protections or claim guaranteed FPS gains.

## Preview
The HTML simulator in `preview/index.html` is for UI testing only. It does not scan, optimize, or modify a device.
