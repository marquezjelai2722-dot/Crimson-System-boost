# Crimson System Boost — Rebuild Architecture

## Product scope
Supported targets: Android APK and Windows desktop EXE. iOS is intentionally out of scope.

This is a modular monolith, not 750 arbitrary folders. Fault containment comes from explicit module APIs, immutable result models, bounded worker execution, permission checks, and independent tests. A folder per tiny file would increase maintenance cost without creating process or security isolation.

## Module boundaries
- Diagnostics: read-only device and OS facts.
- Inventory: installed-app metadata and review classifications.
- Security: platform-native protection status and scan hand-off.
- Cleanup: preview, estimate, confirmation, execute, and report.
- Performance: supported settings, capability detection, apply, rollback.
- Mascot and onboarding: presentation only; never owns privileged operations.

### Android
- `core/model`: immutable results and capability states.
- `core/inventory`: PackageManager metadata inventory; never claims malware verdicts.
- `core/security`: Play Protect / Android security-settings hand-off and optional supported scanner integration. No private-data inspection of other apps.
- `core/cleanup`: only cache or temporary locations exposed by supported APIs; user confirmation before deletion.
- `core/performance`: display mode reporting, user-directed Display settings, and explicitly supported GameManager requests through Shizuku.
- `core/scheduling`: OS-controlled periodic metadata refresh; not a permanent always-running process.
- `ui`: dashboard, app inventory, security, cleanup, performance, settings, onboarding.
- `mascot`: animation and inspection pose, driven by UI state only.

Android limitations:
- An ordinary app cannot read another app's private files or continuously scan every app's runtime memory.
- Package inventory is not antivirus. System/preinstalled status is not a malware verdict.
- The app may show supported display modes and open system display settings. It cannot force a device-wide refresh rate on arbitrary OEM builds.
- Android controls CPU scheduling and thermal limits. Crimson will not attempt to pin all cores awake or override thermal management.

### Windows
- `Core`: result envelope, logging, capability checks.
- `Inventory`: uninstall-registry inventory; never use Win32_Product.
- `Security`: query Microsoft Defender state and request a Defender quick scan when available and authorized.
- `Cleanup`: preview-first cleanup of explicitly allow-listed temporary folders.
- `Performance`: reversible process priority and documented power-plan controls only.
- `UI`: navigation, dashboard, activity feed, and accessible dialogs.
- Each module returns structured status; UI catches failures per feature so one panel cannot crash the shell.

Windows limitations:
- Third-party antivirus engines own their own real-time scanning. Crimson reports available protection and delegates scans to Microsoft Defender when present.
- Core parking is power-plan and firmware dependent. No promise that all cores remain awake; any supported power-plan change must be optional, backed up, and reversible.

## User experience
Visual direction takes interaction cues from Hail and Canta: searchable installed-app lists, clear status labels, explicit per-item actions, and careful confirmation. Crimson retains its own identity, mascot, layout, copy, and visual assets; no source code or branded assets are copied.

Startup:
1. Show a short, skippable loading screen.
2. Mascot enters the inspection-folder pose.
3. Show one useful fact at a time.
4. Run quick read-only diagnostics.
5. Navigate to the dashboard; longer tasks run independently and report progress/errors.

## Classification vocabulary
- Large: only when a trusted storage API provides the measured size.
- Preinstalled: package has system/updated-system metadata.
- Review: heuristic reason to inspect; not a harmful-app verdict.
- Malware: only when a trusted security engine returns a malware detection.
- Protected: platform security is enabled and reports healthy.
- Unknown: unavailable, unsupported, or permission denied.

## Engineering quality gates
- No broad catch that silently hides failure; each feature has a typed result and visible error state.
- No destructive action on app launch or in a background worker.
- Cleanup always previews, calculates size, confirms, and reports skipped files.
- System-setting changes persist a before-state and expose restore.
- Tests cover success, denied permission, unsupported API, timeout, partial failure, and rollback.
- Android and Windows builds run independently; a failure in one platform does not block diagnostics for the other.
- No secrets in source, APK, EXE, logs, or build artifacts.
