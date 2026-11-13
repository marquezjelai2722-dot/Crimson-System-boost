# Crimson System Boost

Crimson System Boost is a native Android utility dashboard. Version **0.2.0** provides live device readings, app discovery/launching, Android settings shortcuts, and an optional Gemini-backed assistant.

## Features in this build

- Live RAM usage and total memory.
- Internal storage free/total capacity.
- Battery percentage, charging state, and Android-reported battery temperature.
- Current display refresh rate.
- Device model, manufacturer, Android version, and build information.
- Installed-app inventory, sorted with user apps first; select an app to launch it or open its Android app-info page.
- Launchable app/game picker.
- Shortcuts to Android battery, display, app management, developer options, storage, network, and Private DNS settings.
- Optional assistant connection to a separately hosted HTTPS backend.

## Build the Android APK

Push to `main` or open **Actions → Build Android APK → Run workflow**. GitHub Actions builds a debug APK, uploads the workflow artifact, and attaches `app-debug.apk` to the existing `App` pre-release.

The APK is a debug build intended for testing/sideloading. It is not a Play Store production release.

## Gemini assistant backend

The app never contains the Gemini API key. The backend reads it from the server environment.

1. Generate a new Gemini API key. Revoke any key previously pasted into a chat or committed to source control.
2. In `backend/`, copy `.env.example` to `.env` and set `GEMINI_API_KEY`.
3. Run with Node.js 20+: `npm start`.
4. Deploy the backend behind HTTPS with authentication, persistent rate limiting, monitoring, and abuse controls before making it public.
5. In the app, enter the HTTPS endpoint ending in `/api/chat`.

The included in-memory IP rate limit is only a basic safeguard. It resets when the server restarts and is not a substitute for production authentication or quotas. Gemini model availability and quotas can change.

## Not included / not claimed

This version does **not** include a real VPN firewall, antivirus/malware detection engine, file quarantine, Shizuku-powered system controls, app freezing, CPU/GPU overclocking, thermal-limit bypass, or guaranteed FPS increases. Those require separate, permission-aware implementations and device testing; the dashboard does not pretend these features are active.

## Safety and privacy

- No blind system-package removal.
- No thermal protection bypass or fake overclock.
- No arbitrary shell command execution.
- Settings changes are made by the user in Android Settings.
- Installed-app visibility depends on Android package-visibility rules and device policy.
- Device readings are informational and can vary by manufacturer.
