# Crimson System Boost

Native Android app with device information, safe settings shortcuts, and a Gemini-backed assistant.

## API key security
Never place a Gemini API key in Android source code, Gradle BuildConfig, an APK, or GitHub. APKs can be inspected. The Android app talks to a separate HTTPS backend; only that backend reads `GEMINI_API_KEY`.

The key previously pasted into chat should be revoked and replaced. Do not reuse it.

## Build the Android APK
Push to `main` or open **Actions → Build Android APK → Run workflow**. The workflow builds a debug APK and uploads `crimson-system-boost-debug`.

## Run the Gemini backend
Requires Node.js 20+ and a newly generated Gemini API key.

1. Copy `backend/.env.example` to `backend/.env`.
2. Set `GEMINI_API_KEY` in `backend/.env`.
3. Run `cd backend && npm start`.
4. Deploy behind HTTPS. Do not expose the raw Node port publicly without TLS/reverse proxy, monitoring, authentication, and abuse controls.
5. In the app, save the HTTPS endpoint ending in `/api/chat`.

The backend includes a simple in-memory per-IP rate limit. For public deployment, add persistent rate limiting, authentication/user quotas, monitoring, and a proper reverse proxy. Gemini free quota is not guaranteed and can change.

## Current app scope
- Reads basic device/build information.
- Opens Android battery and app settings.
- Displays a visible installed-app inventory where Android allows it.
- Sends assistant messages to a configured HTTPS backend.
- Handles connection, quota, and service errors without exposing the API key.

## Not implemented yet
A real VPN firewall, malware detection engine, file quarantine workflow, Shizuku actions, and system-package management are not implemented. No app can guarantee 100% operation on every Android device or network.

## Safety
No blind system-package removal, no thermal bypass, no fake overclock, and no guaranteed FPS claims. File scanning must be limited to user-selected/granted files; destructive actions require clear confirmation.
