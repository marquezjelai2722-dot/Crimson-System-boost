//! Windows diagnostics plus narrowly scoped, user-confirmed maintenance helpers.
use std::{
    io::Read,
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

fn run(program: &str, args: &[&str], timeout: Duration) -> Result<String, String> {
    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Could not start {program}: {e}"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or("Could not capture command output")?;
    let stderr = child
        .stderr
        .take()
        .ok_or("Could not capture command errors")?;
    let stdout_reader = thread::spawn(move || {
        let mut bytes = Vec::new();
        let mut reader = stdout;
        let _ = reader.read_to_end(&mut bytes);
        bytes
    });
    let stderr_reader = thread::spawn(move || {
        let mut bytes = Vec::new();
        let mut reader = stderr;
        let _ = reader.read_to_end(&mut bytes);
        bytes
    });
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if start.elapsed() < timeout => {
                thread::sleep(Duration::from_millis(100));
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return Err(format!("{program} exceeded the diagnostic timeout"));
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return Err(format!("Could not inspect {program}: {error}"));
            }
        }
    };
    let stdout = stdout_reader.join().map_err(|_| "Output reader failed")?;
    let stderr = stderr_reader.join().map_err(|_| "Error reader failed")?;
    let stdout = String::from_utf8_lossy(&stdout).trim().to_owned();
    let stderr = String::from_utf8_lossy(&stderr).trim().to_owned();
    let text = if stdout.is_empty() { stderr } else { stdout };
    if !status.success() {
        return Err(if text.is_empty() {
            format!("{program} exited with status {status}")
        } else {
            text
        });
    }
    if text.is_empty() {
        return Err(format!("{program} returned no diagnostic output"));
    }
    Ok(text)
}
fn bounded(text: String, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text
    } else {
        let shortened: String = text.chars().take(max_chars).collect();
        format!("{shortened}\n\n[Output shortened to keep Crimson responsive.]")
    }
}

/// Lists common per-user and machine startup registry entries without changing them.
#[cfg(windows)]
pub fn startup_inventory() -> Result<String, String> {
    let mut sections = Vec::new();
    for key in [
        r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
        r"HKCU\Software\Microsoft\Windows\CurrentVersion\RunOnce",
        r"HKLM\Software\Microsoft\Windows\CurrentVersion\Run",
        r"HKLM\Software\Microsoft\Windows\CurrentVersion\RunOnce",
    ] {
        let result = Command::new("reg.exe")
            .args(["query", key])
            .output()
            .map_err(|e| format!("Could not query startup entries: {e}"))?;
        let output = String::from_utf8_lossy(&result.stdout).trim().to_owned();
        if result.status.success() && !output.is_empty() {
            sections.push(output);
        } else {
            sections.push(format!("{key}\n  (no entries or inaccessible)"));
        }
    }
    Ok(sections.join("\n\n"))
}

#[cfg(not(windows))]
pub fn startup_inventory() -> Result<String, String> {
    Err("Startup inventory is currently supported on Windows only.".into())
}

/// Returns only direct-child files in the current user's TEMP directory that are at least seven days old.
pub fn scan_cleanup_candidates() -> Result<Vec<PathBuf>, String> {
    use std::time::SystemTime;
    const MAX_ENTRIES: usize = 20_000;
    let root = std::env::var_os("TEMP")
        .or_else(|| std::env::var_os("TMP"))
        .map(PathBuf::from)
        .ok_or("Temporary directory is unavailable")?
        .canonicalize()
        .map_err(|error| format!("Cannot resolve TEMP directory: {error}"))?;
    let cutoff = SystemTime::now()
        .checked_sub(Duration::from_secs(7 * 24 * 60 * 60))
        .ok_or("System clock is invalid")?;
    let mut candidates = Vec::new();
    let entries = std::fs::read_dir(&root)
        .map_err(|error| format!("Cannot inspect {}: {error}", root.display()))?;
    for (index, entry) in entries.enumerate() {
        if index >= MAX_ENTRIES {
            break;
        }
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let kind = match entry.file_type() {
            Ok(kind) => kind,
            Err(_) => continue,
        };
        if !kind.is_file() || kind.is_symlink() {
            continue;
        }
        let metadata = match entry.metadata() {
            Ok(metadata) if metadata.is_file() => metadata,
            _ => continue,
        };
        if metadata
            .modified()
            .map(|modified| modified <= cutoff)
            .unwrap_or(false)
        {
            let path = entry.path();
            if path.parent() == Some(root.as_path()) {
                candidates.push(path);
            }
        }
    }
    Ok(candidates)
}

/// Deletes only explicitly selected, revalidated, seven-day-old direct-child files in TEMP.
pub fn delete_cleanup_candidates(paths: &[PathBuf]) -> Result<String, String> {
    use std::time::SystemTime;
    let root = std::env::var_os("TEMP")
        .or_else(|| std::env::var_os("TMP"))
        .map(PathBuf::from)
        .ok_or("Temporary directory is unavailable")?
        .canonicalize()
        .map_err(|error| format!("Cannot resolve TEMP directory: {error}"))?;
    let cutoff = SystemTime::now()
        .checked_sub(Duration::from_secs(7 * 24 * 60 * 60))
        .ok_or("System clock is invalid")?;
    let mut deleted = 0usize;
    let mut bytes = 0u64;
    let mut skipped = 0usize;
    for path in paths {
        if path.parent() != Some(root.as_path()) {
            skipped += 1;
            continue;
        }
        let kind = match std::fs::symlink_metadata(path) {
            Ok(metadata)
                if metadata.file_type().is_file() && !metadata.file_type().is_symlink() =>
            {
                metadata
            }
            _ => {
                skipped += 1;
                continue;
            }
        };
        let old_enough = kind
            .modified()
            .map(|modified| modified <= cutoff)
            .unwrap_or(false);
        if !old_enough {
            skipped += 1;
            continue;
        }
        match std::fs::remove_file(path) {
            Ok(()) => {
                deleted += 1;
                bytes = bytes.saturating_add(kind.len());
            }
            Err(_) => skipped += 1,
        }
    }
    Ok(format!(
        "Cleanup complete. Deleted {deleted} selected old TEMP files ({:.1} MB). Skipped {skipped} files because they changed, were inaccessible, or were in use. No folders or nested paths were deleted.",
        bytes as f64 / 1_048_576.0
    ))
}

/// Performs bounded DNS and ping diagnostics against a caller-selected hostname.
pub fn network_test(host: &str) -> Result<String, String> {
    let host = host.trim();
    if host.is_empty()
        || host.len() > 253
        || !host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-'))
        || host.starts_with('-')
        || host.ends_with('-')
        || host.contains("..")
    {
        return Err("Enter a valid hostname (letters, numbers, dots and hyphens only).".into());
    }
    #[cfg(windows)]
    let ping_args = vec!["-n", "4", host];
    #[cfg(not(windows))]
    let ping_args = vec!["-c", "4", host];
    let dns = run("nslookup", &[host], Duration::from_secs(8))
        .unwrap_or_else(|e| format!("DNS lookup failed: {e}"));
    let ping = run("ping", &ping_args, Duration::from_secs(10))
        .unwrap_or_else(|e| format!("Ping failed: {e}"));
    Ok(format!(
        "DNS lookup for {host}:\n{dns}\n\nPing test:\n{ping}\n\nDiagnostics only: no adapter, DNS or firewall settings were changed."
    ))
}

/// Lists running processes using Windows tasklist. No process is stopped.
#[cfg(windows)]
pub fn process_inventory() -> Result<String, String> {
    run(
        "tasklist.exe",
        &["/FO", "TABLE", "/NH"],
        Duration::from_secs(6),
    )
    .map(bounded_output)
}

#[cfg(not(windows))]
pub fn process_inventory() -> Result<String, String> {
    Err("Process inventory is currently supported on Windows only.".into())
}

/// Lists installed Windows services. This is an inventory, not a service-control tool.
#[cfg(windows)]
pub fn service_inventory() -> Result<String, String> {
    run(
        "sc.exe",
        &["query", "type=", "service", "state=", "all"],
        Duration::from_secs(8),
    )
    .map(bounded_output)
}

#[cfg(not(windows))]
pub fn service_inventory() -> Result<String, String> {
    Err("Service inventory is currently supported on Windows only.".into())
}

/// Lists uninstall registry records without uninstalling or changing applications.
#[cfg(windows)]
pub fn installed_apps_inventory() -> Result<String, String> {
    let script = "$paths = @('HKLM:\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\*','HKLM:\\Software\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\*','HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\*'); Get-ItemProperty $paths -ErrorAction SilentlyContinue | Where-Object { $_.DisplayName } | Sort-Object DisplayName | Select-Object -First 100 DisplayName, DisplayVersion, Publisher | Format-Table -AutoSize | Out-String -Width 180";
    run(
        "powershell.exe",
        &["-NoProfile", "-NonInteractive", "-Command", script],
        Duration::from_secs(10),
    )
    .map(bounded_output)
}

#[cfg(not(windows))]
pub fn installed_apps_inventory() -> Result<String, String> {
    Err("Installed-app inventory is currently supported on Windows only.".into())
}

/// Reports the active Windows power plan. It does not switch plans or alter power settings.
#[cfg(windows)]
pub fn power_plan() -> Result<String, String> {
    run(
        "powercfg.exe",
        &["/getactivescheme"],
        Duration::from_secs(5),
    )
}

#[cfg(not(windows))]
pub fn power_plan() -> Result<String, String> {
    Err("Power-plan inspection is currently supported on Windows only.".into())
}

/// Reports display adapter names and driver versions through Windows CIM.
#[cfg(windows)]
pub fn graphics_inventory() -> Result<String, String> {
    let script = "Get-CimInstance Win32_VideoController | Select-Object Name, DriverVersion, VideoModeDescription | Format-List | Out-String";
    run(
        "powershell.exe",
        &["-NoProfile", "-NonInteractive", "-Command", script],
        Duration::from_secs(10),
    )
}

#[cfg(not(windows))]
pub fn graphics_inventory() -> Result<String, String> {
    Err("Graphics inventory is currently supported on Windows only.".into())
}

/// Lists signed Plug-and-Play drivers with version and date metadata.
/// This is inventory only: it never downloads or installs a driver.
#[cfg(windows)]
pub fn driver_inventory() -> Result<String, String> {
    let script = "$ErrorActionPreference = 'Stop'; Get-CimInstance Win32_PnPSignedDriver | Where-Object { $_.DeviceName } | Sort-Object DeviceName | Select-Object DeviceName, Manufacturer, DriverVersion, DriverDate, InfName | Format-Table -AutoSize | Out-String -Width 220";
    run(
        "powershell.exe",
        &["-NoProfile", "-NonInteractive", "-Command", script],
        Duration::from_secs(25),
    )
    .map(bounded_output)
}

#[cfg(not(windows))]
pub fn driver_inventory() -> Result<String, String> {
    Err("Driver inventory is currently supported on Windows only.".into())
}

/// Searches the configured Windows Update service for applicable driver updates.
#[cfg(windows)]
pub fn windows_update_driver_inventory() -> Result<String, String> {
    let script = r#"
$ErrorActionPreference = 'Stop'
$session = New-Object -ComObject Microsoft.Update.Session
$searcher = $session.CreateUpdateSearcher()
$result = $searcher.Search("IsInstalled=0 and Type='Driver'")
if ($result.Updates.Count -eq 0) {
    Write-Output 'Windows Update reports no applicable driver updates.'
} else {
    for ($index = 0; $index -lt $result.Updates.Count; $index++) {
        $update = $result.Updates.Item($index)
        $title = ([string]$update.Title) -replace '[\r\n\t]+', ' '
        Write-Output ("{0}. {1}" -f ($index + 1), $title)
    }
}
"#;
    run(
        "powershell.exe",
        &["-NoProfile", "-NonInteractive", "-Command", script],
        Duration::from_secs(120),
    )
    .map(bounded_output)
}

#[cfg(not(windows))]
pub fn windows_update_driver_inventory() -> Result<String, String> {
    Err("Windows Update driver discovery is supported on Windows only.".into())
}

#[cfg(windows)]
pub fn open_optional_driver_updates() -> Result<String, String> {
    Command::new("cmd.exe")
        .args([
            "/C",
            "start",
            "",
            "ms-settings:windowsupdate-optionalupdates",
        ])
        .spawn()
        .map_err(|error| format!("Could not open Windows Update: {error}"))?;
    Ok("Opened Windows Update's optional driver updates page. Review each offered driver before installing.".to_owned())
}

#[cfg(not(windows))]
pub fn open_optional_driver_updates() -> Result<String, String> {
    Err("Windows Update driver controls are supported on Windows only.".into())
}

/// Downloads the latest GitHub release installer only after validating its published SHA-256 manifest.
#[cfg(windows)]
pub fn download_verified_update(current_version: &str) -> Result<Option<PathBuf>, String> {
    if current_version.is_empty()
        || !current_version
            .chars()
            .all(|character| character.is_ascii_digit() || character == '.')
    {
        return Err("The current app version is invalid.".into());
    }

    let script = r#"
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$headers = @{ 'User-Agent' = 'Crimson-System-Boost-Updater' }
$releases = Invoke-RestMethod -Headers $headers -Uri 'https://api.github.com/repos/marquezjelai2722-dot/Crimson-System-boost/releases?per_page=20'
$release = $releases | Where-Object { -not $_.draft } | Sort-Object published_at -Descending | Select-Object -First 1
if ($null -eq $release) { throw 'No published GitHub release was found.' }
$tag = [string]$release.tag_name
if ($tag -notmatch '^v\d+\.\d+\.\d+([-.][0-9A-Za-z.-]+)?$') { throw 'The release tag has an invalid version format.' }
$latestVersion = [version]($tag.Substring(1).Split('-')[0])
$currentVersion = [version]'__CURRENT_VERSION__'
if ($latestVersion -le $currentVersion) {
    Write-Output ('NO_UPDATE|' + $tag)
    exit 0
}
$versionText = $tag.Substring(1)
$installerName = 'Crimson-System-Boost-Setup-v' + $versionText + '.exe'
$installerAsset = $release.assets | Where-Object { $_.name -eq $installerName } | Select-Object -First 1
$manifestAsset = $release.assets | Where-Object { $_.name -eq 'SHA256SUMS.txt' } | Select-Object -First 1
if ($null -eq $installerAsset -or $null -eq $manifestAsset) { throw 'Release is missing the installer or SHA256SUMS.txt.' }
$destination = Join-Path $env:TEMP ('Crimson-System-Boost-' + $tag + '-Setup.exe')
$manifestPath = Join-Path $env:TEMP ('Crimson-System-Boost-' + $tag + '-SHA256SUMS.txt')
Invoke-WebRequest -Headers $headers -Uri $installerAsset.browser_download_url -OutFile $destination
Invoke-WebRequest -Headers $headers -Uri $manifestAsset.browser_download_url -OutFile $manifestPath
$manifest = Get-Content -LiteralPath $manifestPath
$line = $manifest | Where-Object { $_ -match ('^[A-Fa-f0-9]{64}\s+\*?' + [regex]::Escape($installerName) + '$') } | Select-Object -First 1
if ($null -eq $line) { Remove-Item -LiteralPath $destination -Force; throw 'Installer checksum entry was not found.' }
$expected = ($line -split '\s+')[0].ToLowerInvariant()
$actual = (Get-FileHash -LiteralPath $destination -Algorithm SHA256).Hash.ToLowerInvariant()
if ($actual -ne $expected) { Remove-Item -LiteralPath $destination -Force; throw 'Installer SHA-256 mismatch. The downloaded file was deleted.' }
Write-Output ('DOWNLOADED|' + $destination)
"#.replace("__CURRENT_VERSION__", current_version);

    let output = run(
        "powershell.exe",
        &[
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ],
        Duration::from_secs(180),
    )?;
    if output.starts_with("NO_UPDATE|") {
        return Ok(None);
    }
    let path = output
        .strip_prefix("DOWNLOADED|")
        .ok_or_else(|| format!("Updater returned an unexpected result: {output}"))?;
    let installer = PathBuf::from(path.trim());
    if !installer.is_file() {
        return Err("The verified installer could not be found after download.".into());
    }
    Ok(Some(installer))
}

#[cfg(not(windows))]
pub fn download_verified_update(_current_version: &str) -> Result<Option<PathBuf>, String> {
    Err("The Crimson self-updater is supported on Windows only.".into())
}

/// Runs non-repairing Windows image and protected-file checks with explicit UAC elevation.
#[cfg(windows)]
pub fn system_health_scan() -> Result<String, String> {
    let script = r#"
$ErrorActionPreference = 'Stop'
$scriptPath = Join-Path $env:TEMP ('CrimsonHealth-' + [guid]::NewGuid().ToString('N') + '.ps1')
$logPath = Join-Path $env:TEMP ('CrimsonHealth-' + [guid]::NewGuid().ToString('N') + '.log')
$child = @'
param([string]$LogPath)
$ErrorActionPreference = 'Continue'
& DISM.exe /Online /Cleanup-Image /ScanHealth *>&1 | Out-File -LiteralPath $LogPath -Append -Encoding utf8
$dismCode = $LASTEXITCODE
if ($dismCode -ne 0) { exit $dismCode }
& sfc.exe /verifyonly *>&1 | Out-File -LiteralPath $LogPath -Append -Encoding utf8
exit $LASTEXITCODE
'@
Set-Content -LiteralPath $scriptPath -Value $child -Encoding utf8
try {
    $arguments = '-NoProfile -File "' + $scriptPath + '" -LogPath "' + $logPath + '"'
    $process = Start-Process -FilePath 'powershell.exe' -Verb RunAs -ArgumentList $arguments -Wait -PassThru
    if (Test-Path -LiteralPath $logPath) {
        Get-Content -LiteralPath $logPath -Raw
    }
    if ($process.ExitCode -ne 0) { exit $process.ExitCode }
} finally {
    Remove-Item -LiteralPath $scriptPath -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $logPath -Force -ErrorAction SilentlyContinue
}
"#;
    run(
        "powershell.exe",
        &["-NoProfile", "-NonInteractive", "-Command", script],
        Duration::from_secs(600),
    )
}

#[cfg(not(windows))]
pub fn system_health_scan() -> Result<String, String> {
    Err("Windows component health scans are supported on Windows only.".into())
}

#[cfg(windows)]
pub fn launch_system_repair() -> Result<String, String> {
    let script = "$arguments = '/c DISM.exe /Online /Cleanup-Image /RestoreHealth && sfc.exe /scannow'; $process = Start-Process -FilePath 'cmd.exe' -Verb RunAs -ArgumentList $arguments -Wait -PassThru; if ($process.ExitCode -ne 0) { exit $process.ExitCode }";
    Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .spawn()
        .map_err(|error| format!("Could not request elevated Windows repair: {error}"))?;
    Ok("Requested an administrator repair window. Approve the Windows UAC prompt to run DISM RestoreHealth followed by SFC /scannow. Review the command window and restart if Windows requests it.".to_owned())
}

#[cfg(not(windows))]
pub fn launch_system_repair() -> Result<String, String> {
    Err("Windows component repair is supported on Windows only.".into())
}

fn bounded_output(text: String) -> String {
    bounded(text, 12_000)
}

#[cfg(test)]
mod tests {
    #[test]
    fn rejects_invalid_network_hosts() {
        assert!(super::network_test("bad host").is_err());
        assert!(super::network_test("").is_err());
        assert!(super::network_test("-bad.example").is_err());
        assert!(super::network_test("bad..example").is_err());
    }

    #[test]
    fn output_limit_is_applied() {
        let result = super::bounded("x".repeat(20), 10);
        assert!(result.len() < 100);
        assert!(result.contains("Output shortened"));
    }
}
