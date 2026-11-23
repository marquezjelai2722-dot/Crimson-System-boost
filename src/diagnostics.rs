//! Read-only Windows diagnostics. These commands inspect state; they never modify it.
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

/// Scans a bounded portion of the current user's TEMP tree without deleting anything.
pub fn cleanup_preview() -> Result<String, String> {
    const MAX_ENTRIES: usize = 20_000;
    const MAX_DEPTH: usize = 4;

    let temp = std::env::var_os("TEMP")
        .or_else(|| std::env::var_os("TMP"))
        .map(PathBuf::from)
        .ok_or("Temporary directory is unavailable")?;
    let mut pending = vec![(temp.clone(), 0usize)];
    let mut files = 0u64;
    let mut directories = 0u64;
    let mut bytes = 0u64;
    let mut skipped = 0u64;
    let mut inspected = 0usize;

    while let Some((directory, depth)) = pending.pop() {
        let entries = match std::fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(_) => {
                skipped += 1;
                continue;
            }
        };

        for entry in entries {
            if inspected >= MAX_ENTRIES {
                break;
            }
            inspected += 1;
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => {
                    skipped += 1;
                    continue;
                }
            };
            let kind = match entry.file_type() {
                Ok(kind) => kind,
                Err(_) => {
                    skipped += 1;
                    continue;
                }
            };

            if kind.is_symlink() {
                skipped += 1;
            } else if kind.is_file() {
                match entry.metadata() {
                    Ok(metadata) => {
                        files += 1;
                        bytes = bytes.saturating_add(metadata.len());
                    }
                    Err(_) => skipped += 1,
                }
            } else if kind.is_dir() && depth < MAX_DEPTH {
                directories += 1;
                pending.push((entry.path(), depth + 1));
            } else {
                skipped += 1;
            }
        }
        if inspected >= MAX_ENTRIES {
            break;
        }
    }

    Ok(format!(
        "Temporary folder: {}\nFiles inspected: {files}\nSubfolders inspected: {directories}\nPotential file size: {:.1} MB\nSkipped / inaccessible entries: {skipped}\n\nPreview only: nothing was deleted. Scan limits: {MAX_ENTRIES} entries and {MAX_DEPTH} folder levels. Recheck files before removal; some temporary files may be in use.",
        temp.display(), bytes as f64 / 1_048_576.0
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

/// Runs non-repairing Windows image and protected-file checks.
/// DISM ScanHealth and SFC verifyonly inspect; they do not repair or replace files.
#[cfg(windows)]
pub fn system_health_scan() -> Result<String, String> {
    let dism = run(
        "DISM.exe",
        &["/Online", "/Cleanup-Image", "/ScanHealth"],
        Duration::from_secs(240),
    )?;
    let sfc = run("sfc.exe", &["/verifyonly"], Duration::from_secs(240))?;
    Ok(format!(
        "DISM component-store scan:\n{dism}\n\nSystem File Checker verification:\n{sfc}\n\nScan only: no repairs or file replacements were performed. If corruption is reported, review the result before choosing a repair."
    ))
}

#[cfg(not(windows))]
pub fn system_health_scan() -> Result<String, String> {
    Err("Windows component health scans are supported on Windows only.".into())
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
