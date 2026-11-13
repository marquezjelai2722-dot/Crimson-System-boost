@echo off
setlocal
cd /d "%~dp0"
powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -STA -File "%~dp0Crimson-Windows-GUI.ps1"
if errorlevel 1 (
  echo.
  echo Crimson System Boost closed with an error.
  pause
)
endlocal
