//! Read-only emergency detection and supported Windows maintenance entry points.
//! Driver packages are never deleted or replaced automatically. Updates are handed to
//! Windows Update so the user can review applicable signed packages.

#[cfg(windows)]
use std::process::Command;

#[derive(Clone, Debug, Default)]
pub struct EmergencyReport {
    pub issues: Vec<String>,
    pub notes: Vec<String>,
    pub summary: String,
}

impl EmergencyReport {
    pub fn has_urgent_issues(&self) -> bool {
        !self.issues.is_empty()
    }
}

#[cfg(windows)]
fn run_powershell(script: &str) -> Result<String, String> {
    let output = Command::new("powershell.exe")
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            script,
        ])
        .output()
        .map_err(|error| format!("Could not start Windows maintenance check: {error}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    if !output.status.success() {
        return Err(if stderr.is_empty() {
            format!("Windows maintenance check exited with {}", output.status)
        } else {
            stderr
        });
    }
    Ok(if stdout.is_empty() {
        "Windows returned no diagnostic details.".to_owned()
    } else {
        stdout
    })
}

/// Checks for Plug-and-Play devices with a non-zero Configuration Manager error
/// and recent Windows Update error events. It does not modify the system.
#[cfg(windows)]
pub fn scan_emergency_conditions() -> Result<EmergencyReport, String> {
    let script = r#"
$ErrorActionPreference = 'Stop'
$issues = @()
$notes = @()
try {
    $devices = @(Get-CimInstance -ClassName Win32_PnPEntity |
        Where-Object { $null -ne $_.ConfigManagerErrorCode -and $_.ConfigManagerErrorCode -ne 0 } |
        Select-Object -First 8)
    foreach ($device in $devices) {
        $name = ([string]$device.Name) -replace '[\r\n|]+', ' '
        $code = [int]$device.ConfigManagerErrorCode
        $issues += ('ISSUE|Device problem code {0}: {1}' -f $code, $name)
    }
} catch {
    Write-Output ('CHECK_WARNING|Could not inspect Plug-and-Play devices: ' + $_.Exception.Message)
}
try {
    $since = (Get-Date).AddDays(-14)
    $events = @(Get-WinEvent -FilterHashtable @{
        LogName = 'Microsoft-Windows-WindowsUpdateClient/Operational'
        StartTime = $since
        Level = 2
    } -MaxEvents 6 -ErrorAction Stop)
    foreach ($event in $events) {
        $message = ([string]$event.Message -replace '[\r\n|]+', ' ')
        if ($message.Length -gt 220) { $message = $message.Substring(0, 220) }
        $notes += ('NOTE|Recent Windows Update error event {0} ({1}): {2}' -f $event.Id, $event.TimeCreated, $message)
    }
} catch {
    # An unavailable or empty event log is not itself evidence of a broken updater.
}
try {
    $os = Get-CimInstance -ClassName Win32_OperatingSystem
    if ([string]$os.Caption -match 'Windows 10') {
        Write-Output 'NOTE|Windows 10 support ended on 14 October 2025. Security updates after that date require eligible Windows 10 ESU enrollment.'
    }
} catch { }
if ($issues.Count -eq 0) {
    Write-Output 'OK|No Plug-and-Play device error codes were detected.'
}
$issues | ForEach-Object { Write-Output $_ }
$notes | ForEach-Object { Write-Output $_ }
"#;
    let output = run_powershell(script)?;
    let mut issues = Vec::new();
    let mut notes = Vec::new();
    let mut warnings = Vec::new();
    let mut summary = String::new();

    for line in output.lines() {
        if let Some(issue) = line.strip_prefix("ISSUE|") {
            issues.push(issue.trim().to_owned());
        } else if let Some(note) = line.strip_prefix("NOTE|") {
            notes.push(note.trim().to_owned());
        } else if let Some(warning) = line.strip_prefix("CHECK_WARNING|") {
            warnings.push(warning.trim().to_owned());
        } else if let Some(ok) = line.strip_prefix("OK|") {
            summary = ok.trim().to_owned();
        }
    }

    if !warnings.is_empty() {
        summary.push_str(if summary.is_empty() { "" } else { "\n" });
        summary.push_str(&warnings.join("\n"));
    }
    if summary.is_empty() && !issues.is_empty() {
        summary = format!(
            "Detected {} device problem code(s) requiring review.",
            issues.len()
        );
    }
    if summary.is_empty() && issues.is_empty() {
        summary =
            "The emergency scan completed, but Windows returned no recognizable status.".to_owned();
    }
    Ok(EmergencyReport {
        issues,
        notes,
        summary,
    })
}

#[cfg(not(windows))]
pub fn scan_emergency_conditions() -> Result<EmergencyReport, String> {
    Err("Emergency Windows checks are supported on Windows only.".into())
}

/// Requests the normal Windows Automatic Updates detection task and opens Settings.
/// Windows controls the download/install policy; Crimson does not bypass it.
#[cfg(windows)]
pub fn request_windows_update_detection() -> Result<String, String> {
    let script = r#"
$ErrorActionPreference = 'Stop'
$automaticUpdates = New-Object -ComObject Microsoft.Update.AutoUpdate
$automaticUpdates.DetectNow()
Start-Process 'ms-settings:windowsupdate'
Write-Output 'Requested Windows Automatic Updates detection and opened Windows Update. Review available updates and approve installation or restart in Windows Settings.'
"#;
    run_powershell(script)
}

#[cfg(not(windows))]
pub fn request_windows_update_detection() -> Result<String, String> {
    Err("Windows Update is supported on Windows only.".into())
}

/// Re-enumerates Plug-and-Play devices with the built-in PnPUtil tool.
/// It does not remove drivers or force-install a package.
#[cfg(windows)]
pub fn request_device_rescan() -> Result<String, String> {
    let script = r#"
$ErrorActionPreference = 'Stop'
$process = Start-Process -FilePath "$env:WINDIR\System32\pnputil.exe" -ArgumentList '/scan-devices' -Verb RunAs -Wait -PassThru
if ($process.ExitCode -ne 0) { throw ('PnPUtil device rescan failed with exit code ' + $process.ExitCode) }
Write-Output 'Device re-enumeration completed. Re-scan the device list, then check Windows Update for applicable signed drivers.'
"#;
    run_powershell(script)
}

#[cfg(not(windows))]
pub fn request_device_rescan() -> Result<String, String> {
    Err("Plug-and-Play device rescanning is supported on Windows only.".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_only_flags_nonempty_issue_list() {
        let clean = EmergencyReport {
            issues: Vec::new(),
            notes: Vec::new(),
            summary: "No issues".to_owned(),
        };
        let urgent = EmergencyReport {
            issues: vec!["A device reports a problem code".to_owned()],
            notes: Vec::new(),
            summary: String::new(),
        };
        assert!(!clean.has_urgent_issues());
        assert!(urgent.has_urgent_issues());
    }
}
