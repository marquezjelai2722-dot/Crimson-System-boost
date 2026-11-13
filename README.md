# Crimson System Boost

Crimson System Boost is a safety-first performance toolkit for Android phones/tablets and Windows PCs/laptops. It combines local diagnostics, reversible gaming profiles, safe cleanup, security checks, and optional assistant support.

## Current Android app (v0.3.0 baseline)
- Live RAM, storage, battery, charging state, battery temperature, display refresh rate, device model and Android version.
- Local launcher for installed apps and games.
- Optional HTTPS assistant backend; provider API keys remain server-side.
- Optional Shizuku operations with explicit permission and confirmation:
  - Android GameManager performance/standard mode requests for a selected game.
  - Package Manager reclaimable cache trim.
  - ART speed-profile compilation request for a selected app.

## Cross-platform direction
- **Android:** native app with hardware/display diagnostics and supported GameManager requests.
- **Windows PCs and laptops:** PowerShell companion for hardware/display diagnostics, reversible game-process priority profiles, restore, and safe temporary-file cleanup.
- **Shared FPS dashboard direction:** distinguish display refresh rate (Hz), game FPS, and frame pacing. Supported settings only; no universal cap bypass is possible.

### Windows companion
Requires Windows 10/11 and PowerShell 5.1+. From the repository folder:

```powershell
Set-ExecutionPolicy -Scope Process Bypass
.\windows\Crimson-Windows.ps1
```

Actions: `-Action Scan`, `Boost`, `Restore`, `CleanPreview`, or `Clean`.
Boost requests AboveNormal priority for a selected visible process and saves its previous priority. Cleaner targets only files older than seven days in the current user's temp and Windows temp folders; preview first, then type DELETE to confirm. Locked files are skipped.

## FPS and safety limits
Crimson aims for the highest stable FPS supported by hardware, display, OS, and game. It cannot guarantee an FPS increase or universally remove a game's internal frame cap. It does not patch game files, bypass anti-cheat, disable thermal protections, overclock hardware, use Realtime process priority, or run arbitrary shell commands. See [FPS controls](docs/FPS-CONTROL-AND-UNCAPPING.md).

## Security
See [SECURITY.md](SECURITY.md). CodeQL and dependency review workflows run on repository changes. Never commit API keys, signing keys, tokens, or production .env files. Revoke credentials previously exposed in chat or source control.

## Build Android APK
Push to main or open Actions → Build Android APK → Run workflow. GitHub Actions builds a debug APK and uploads a temporary workflow artifact. The APK is debug-signed for direct testing/sideloading, not Play Store production.

## Assistant backend and API keys
The app does not contain a Gemini API key. The backend reads its key from a server environment variable.
1. Generate a new API key and revoke any key previously pasted into chat or source control.
2. In backend/, copy .env.example to .env and set GEMINI_API_KEY.
3. Run with Node.js 20+: npm start.
4. Before public deployment, add authentication, persistent rate limiting, monitoring, and abuse controls.
5. In the app, enter the HTTPS endpoint ending in /api/chat.

Never embed a provider API key in the APK. A .gitignore entry does not protect a key shipped inside an Android package.

## Android compatibility
- Shizuku API 13.1.5; Android 8.0+ minimum app version.
- Advanced commands depend on Android build and OEM implementation.
- Shizuku via ADB has shell identity (UID 2000), not unrestricted root.
- Hardware readings may be unavailable or vendor-specific.
