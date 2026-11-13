# Crimson System Boost

Crimson System Boost is a native Android performance utility with a restrained dark interface, live hardware monitoring, local app tools, and optional Shizuku-powered advanced operations.

## Version 0.3.0

### Core features
- Live RAM, storage, battery, charging state, battery temperature, display refresh rate, device model and Android version.
- Local launcher for installed apps and games.
- Optional HTTPS assistant backend. Provider API keys remain server-side.

### Advanced mode (Shizuku required)
Advanced operations run in a Shizuku UserService with the shell/ADB identity. The app requests Shizuku permission and asks for confirmation before each operation.

- **GameManager mode:** request performance mode or restore standard mode for a selected game. Availability depends on Android/OEM support and game classification.
- **System cache maintenance:** request Package Manager to trim reclaimable app caches toward a 256 MB free-space target. It does not target app data or personal files; cached content may be rebuilt.
- **ART profile compilation:** request speed-profile compilation for one selected app. This may take time, use storage, or be unsupported.

The app exposes a small allow-list of fixed operations. It does not accept arbitrary shell commands, remove system packages, disable thermal protections, or overclock CPU/GPU hardware. Android can reject or ignore operations, and no FPS/performance increase is guaranteed.

## Shizuku setup
1. Install Shizuku from its official download page: https://shizuku.rikka.app/download/
2. Start Shizuku using Wireless debugging (Android 11+) or a supported root setup.
3. Open Crimson System Boost and choose **Connect Shizuku**.
4. Approve the permission request in Shizuku.
5. Run an advanced action and review its confirmation prompt.

On non-rooted devices, Shizuku started through Wireless debugging normally needs to be restarted after a reboot. Crimson does not silently start Shizuku or bypass its authorization.

## Build the Android APK
Push to `main` or open **Actions → Build Android APK → Run workflow**. GitHub Actions builds a debug APK, uploads a temporary workflow artifact, and publishes a versioned APK to the repository's **Releases** page.

The APK is debug-signed for direct testing/sideloading. It is not a Play Store production-signed build.

## Assistant backend and API keys
The app does not contain a Gemini API key. The backend reads its key from a server environment variable.

1. Generate a new API key and revoke any key previously pasted into chat or source control.
2. In `backend/`, copy `.env.example` to `.env` and set `GEMINI_API_KEY`.
3. Run with Node.js 20+: `npm start`.
4. Before public deployment, add authentication, persistent rate limiting, monitoring and abuse controls.
5. In the app, enter the HTTPS endpoint ending in `/api/chat`.

Never embed a provider API key in the APK. A `.gitignore` entry does not protect a key shipped inside an Android package.

## Compatibility and limitations
- Shizuku API 13.1.5; Android 8.0+ minimum app version.
- Advanced commands depend on the device's Android build and OEM implementation.
- Shizuku started through ADB has shell identity (UID 2000), not unrestricted root.
- No thermal protection bypass, arbitrary shell input, blind package removal, or guaranteed FPS claims.
- Battery temperature and other hardware readings are informational and can be unavailable or vendor-specific.
