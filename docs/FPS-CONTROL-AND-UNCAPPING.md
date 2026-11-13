# FPS controls and uncapping

Crimson targets the highest stable frame rate supported by hardware, display, OS, and game. It cannot guarantee an FPS increase or universally remove a game's own frame cap.

## Android
- Report current and supported display refresh modes.
- Request Android GameManager performance mode for a selected game when supported (existing Shizuku allow-listed feature).
- Only request per-app refresh behavior through documented Android APIs.
- Never modify another app's private config, patch game binaries, bypass anti-cheat, or disable thermal management.
- A 90 Hz panel does not mean every game supports 90 FPS.

## Windows PCs and laptops
- Report active display mode, GPU, driver, and thermal/power constraints.
- Offer a reversible per-process AboveNormal priority profile (Windows companion).
- Future native desktop UI can expose supported display modes and per-game settings with backups and rollback.
- Never use Realtime priority, undocumented registry tweaks, forced driver flags, or anti-cheat bypasses.

## Terms
- Refresh rate (Hz): display refreshes per second.
- FPS: frames rendered by the game per second.
- Frame pacing: consistency of frame delivery.
- Uncap: remove a software-imposed limit only through a supported game/OS setting or API. It cannot overcome hardware, engine, network, thermal, or display limits.

## Acceptance checks
1. Diagnostics report actual supported/active modes, not invented FPS values.
2. Every system-changing action explains trade-offs and requires confirmation.
3. Restore returns process priorities to saved values.
4. Cleaner previews files and requires explicit typed confirmation.
5. No guaranteed-FPS, thermal-bypass, overclocking, or anti-cheat claims.
