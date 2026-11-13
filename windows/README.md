# Crimson System Boost for Windows

## Requirements
- Windows 10 or Windows 11
- Windows PowerShell 5.1 and .NET Framework WinForms (included on supported Windows installations)
- Normal user account for the included features

## Launch
Extract this folder and double-click Run-Crimson-Windows.bat. Alternatively, open PowerShell in this folder and run:

    powershell.exe -NoProfile -ExecutionPolicy Bypass -STA -File .\Crimson-Windows-GUI.ps1

Do not bypass security policy on a managed school or work device; ask its administrator.

## Features
- System overview: Windows version, CPU, installed RAM, first detected GPU, driver-reported display refresh rate, and system-drive free space.
- Game profile: select a visible running process and request AboveNormal priority. The previous priority is saved in %LOCALAPPDATA%\CrimsonSystemBoost\process-priority-backup.json.
- Restore: restore saved priority for the same process ID and name if it is still running.
- Cleaner preview: lists temporary files older than seven days in the current user's temp folder and Windows temp folder.
- Cleaner: asks for confirmation before deleting only eligible files. Locked or inaccessible files are skipped.

## Safety and limitations
No registry modifications, service disabling, driver changes, overclocking, Realtime priority, anti-cheat bypass, or thermal-protection changes. A process-priority request does not guarantee higher FPS. Display refresh rate is not a measurement of game FPS. This utility does not scan for malware or claim that a PC is clean. The cleaner does not recurse into folders or empty the Recycle Bin.

This is a PowerShell desktop utility, not a signed native installer or EXE package.
