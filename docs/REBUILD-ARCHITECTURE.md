# Crimson System Boost — Modular Rebuild

## Product scope
- Android APK and Windows EXE only. iOS is intentionally unsupported.
- Original UI inspired by the clarity of Hail's reversible app-state controls and Canta's searchable app inventory, without copying their code, assets, or branding.
- Dark Crimson interface, restrained red/cyan accents, readable text, mascot inspection/loading state.

## Reliability model
Do not create hundreds of empty folders. Use bounded modules with narrow contracts:
- Android: UI, domain, inventory, security, maintenance, performance, platform adapters.
- Windows: UI, inventory, security, maintenance, performance, shared diagnostics.
- A module returns typed results; it never terminates the host UI process.
- Each app/package/file is processed independently. A single access-denied item becomes a warning, not a failed full scan.
- Destructive actions require preview + explicit confirmation. Reversible changes store a backup before applying.
- Background jobs are scheduled by the OS, cancellable, rate-limited, and never run an unbounded tight loop.

## Truthful scan semantics
### Android
The app can enumerate visible installed package metadata and APK file size. It cannot read every app's private data or independently prove that an installed package is malware. Malware verdicts must come from Android Play Protect or a trusted signature/reputation engine. Inventory entries therefore use NOT_SCANNED and do not show a fake green "clean" verdict. A periodic WorkManager job refreshes inventory metadata only; it is not an antivirus scanner.

### Windows
Use Microsoft Defender's own status, detections, and scan engine when available. Never present resource usage or install size as proof of malware. Resource-heavy processes and large installed packages are separate informational categories. If Defender is disabled or unavailable, report that explicitly.

## Supported controls
- Temporary-file cleanup: preview first, age/target limits, per-file exception handling, never target user documents or app data.
- Refresh rate: show reported modes and open the OS display settings. Do not promise a global refresh-rate override when the OEM/driver does not expose one.
- CPU: show core/thread topology and use supported OS power modes. Do not disable thermal protections, force all cores awake, or disable core parking/idle states; those can increase heat and wear without improving game frame pacing.
- Android advanced operations remain optional and permission-gated through Shizuku. No root assumption.

## Folder boundaries
    app/src/main/java/com/crimson/boost/
      ui/                 Activity presentation and reusable view builders
      core/inventory/     Installed-app metadata and periodic inventory refresh
      core/security/      Verdict adapters; "unknown" is not "safe"
      core/maintenance/   Preview-first cleanup contracts
      core/performance/   Supported game/display/power requests
      platform/android/   Android and Shizuku adapters
    windows/
      modules/            Inventory, security, maintenance, performance
      ui/                 WinForms composition
      Build-Windows.ps1   Bundles modules into the distributable EXE

## Failure containment
- Catch exceptions at package/file/process boundaries.
- Return structured result objects with counts and warnings.
- Never swallow a whole-job failure as success.
- Keep the UI responsive by running scans off the UI thread.
- Persist only small scan summaries and reversible-setting backups.
- Do not upload app inventories or file paths by default.

## Acceptance checks
1. One unreadable package does not abort the inventory scan.
2. One locked temp file does not abort cleanup of other eligible files.
3. A missing Defender cmdlet is reported as unavailable, not as "no threats".
4. Cancelled cleanup leaves files untouched.
5. A failed Shizuku connection does not crash the Android UI.
6. Android background inventory refresh is OS-scheduled and does not claim antivirus coverage.
7. Display settings open the OS settings page; no unsupported global refresh-rate write is attempted.
8. No iOS target, target-specific source, or iOS release workflow is produced.
