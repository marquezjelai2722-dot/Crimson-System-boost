# Crimson System Boost - Windows companion
# Windows 10/11 PowerShell 5.1+. Run as a normal user.
# No registry tweaks, service disabling, driver changes, overclocking, or anti-cheat bypass.

[CmdletBinding()]
param([ValidateSet("Menu","Scan","Boost","Restore","CleanPreview","Clean")][string]$Action = "Menu")

$ErrorActionPreference = "Stop"
$StateDir = Join-Path $env:LOCALAPPDATA "CrimsonSystemBoost"
$StateFile = Join-Path $StateDir "process-priority-backup.json"

function Scan-System {
    Write-Host "=== CRIMSON SYSTEM SCAN ===" -ForegroundColor Cyan
    Get-CimInstance Win32_OperatingSystem | Select-Object Caption, Version, OSArchitecture | Format-List
    Get-CimInstance Win32_Processor | Select-Object Name, NumberOfCores, NumberOfLogicalProcessors | Format-List
    Get-CimInstance Win32_PhysicalMemory | Measure-Object Capacity -Sum |
        ForEach-Object { "Installed RAM: {0:N1} GB" -f ($_.Sum / 1GB) }
    Get-Volume | Where-Object DriveLetter |
        Select-Object DriveLetter, FileSystemLabel,
        @{N="FreeGB";E={[math]::Round($_.SizeRemaining/1GB,1)}},
        @{N="SizeGB";E={[math]::Round($_.Size/1GB,1)}} | Format-Table -AutoSize
    Get-CimInstance Win32_VideoController |
        Select-Object Name, CurrentRefreshRate, DriverVersion | Format-Table -AutoSize
    Write-Host "Refresh rate is not the same as game FPS."
}

function Boost-Game {
    Write-Host "=== REVERSIBLE GAME PROFILE ===" -ForegroundColor Cyan
    Get-Process | Where-Object { $_.MainWindowTitle } |
        Select-Object Id, ProcessName, MainWindowTitle | Format-Table -AutoSize
    $name = Read-Host "Exact process name (without .exe)"
    if ([string]::IsNullOrWhiteSpace($name)) { return }
    $targets = @(Get-Process -Name $name -ErrorAction SilentlyContinue)
    if (-not $targets) { Write-Warning "Process not found."; return }
    $backup = foreach ($p in $targets) {
        [pscustomobject]@{ Id=$p.Id; Name=$p.ProcessName; Priority=$p.PriorityClass.ToString() }
    }
    New-Item -ItemType Directory -Force -Path $StateDir | Out-Null
    $backup | ConvertTo-Json -Depth 4 | Set-Content -Encoding UTF8 $StateFile
    foreach ($p in $targets) {
        try { $p.PriorityClass = [System.Diagnostics.ProcessPriorityClass]::AboveNormal }
        catch { Write-Warning "Could not change $($p.ProcessName): $($_.Exception.Message)" }
    }
    Write-Host "AboveNormal requested where permitted. No FPS gain is guaranteed."
}

function Restore-Game {
    if (-not (Test-Path $StateFile)) { Write-Host "No saved profile."; return }
    foreach ($entry in @(Get-Content $StateFile -Raw | ConvertFrom-Json)) {
        try {
            $p = Get-Process -Id ([int]$entry.Id) -ErrorAction Stop
            if ($p.ProcessName -eq $entry.Name) {
                $p.PriorityClass = [System.Diagnostics.ProcessPriorityClass]::$($entry.Priority)
                Write-Host "Restored $($entry.Name) ($($entry.Id))"
            }
        } catch { Write-Warning "Skipped PID $($entry.Id); process exited or changed." }
    }
}

function Get-TempTargets {
    $roots = @($env:TEMP, "$env:WINDIR\Temp") | Where-Object { $_ -and (Test-Path $_) } | Select-Object -Unique
    foreach ($root in $roots) {
        Get-ChildItem -LiteralPath $root -Force -File -ErrorAction SilentlyContinue |
            Where-Object { $_.LastWriteTime -lt (Get-Date).AddDays(-7) }
    }
}

function Clean-Temp([bool]$PreviewOnly) {
    Write-Host "=== SAFE TEMP CLEANER ===" -ForegroundColor Cyan
    $items = @(Get-TempTargets)
    $bytes = ($items | Measure-Object Length -Sum).Sum
    if (-not $bytes) { $bytes = 0 }
    Write-Host ("Eligible files: {0}; potential space: {1:N1} MB" -f $items.Count, ($bytes/1MB))
    $items | Select-Object -First 30 FullName, @{N="MB";E={[math]::Round($_.Length/1MB,2)}}, LastWriteTime | Format-Table -AutoSize
    if ($items.Count -gt 30) { Write-Host ("... plus {0} more files" -f ($items.Count-30)) }
    if ($PreviewOnly) { Write-Host "Preview only. Nothing deleted."; return }
    if ((Read-Host "Delete only these files older than 7 days? Type DELETE") -cne "DELETE") {
        Write-Host "Cancelled."; return
    }
    $removed = 0
    foreach ($item in $items) {
        try { Remove-Item -LiteralPath $item.FullName -Force -ErrorAction Stop; $removed++ }
        catch { Write-Warning "Skipped locked/inaccessible file: $($item.Name)" }
    }
    Write-Host "Removed $removed temporary files."
}

switch ($Action) {
    "Scan" { Scan-System }
    "Boost" { Boost-Game }
    "Restore" { Restore-Game }
    "CleanPreview" { Clean-Temp $true }
    "Clean" { Clean-Temp $false }
    default {
        Write-Host "CRIMSON SYSTEM BOOST - WINDOWS" -ForegroundColor Cyan
        Write-Host "1 Scan  2 Apply game profile  3 Restore  4 Cleaner preview  5 Clean"
        switch (Read-Host "Choose 1-5") {
            "1" { Scan-System } "2" { Boost-Game } "3" { Restore-Game }
            "4" { Clean-Temp $true } "5" { Clean-Temp $false }
            default { Write-Host "No action selected." }
        }
    }
}
