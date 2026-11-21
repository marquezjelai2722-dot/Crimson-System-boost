//! Read-only Windows diagnostics. These commands inspect state; they never modify it.
use std::{path::PathBuf, process::{Command, Stdio}, time::{Duration, Instant}};

fn run(program: &str, args: &[&str], timeout: Duration) -> Result<String, String> {
    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Could not start {program}: {e}"))?;
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let output = child.wait_with_output().map_err(|e| e.to_string())?;
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
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
                return Ok(text);
            }
            Ok(None) if start.elapsed() < timeout => std::thread::sleep(Duration::from_millis(100)),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("{program} exceeded the diagnostic timeout"));
            }
            Err(e) => return Err(format!("Could not inspect {program}: {e}")),
        }
    }
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

/// Measures candidate temporary files. It deliberately does not delete anything.
pub fn cleanup_preview() -> Result<String, String> {
    let temp = std::env::var_os("TEMP")
        .or_else(|| std::env::var_os("TMP"))
        .map(PathBuf::from)
        .ok_or("Temporary directory is unavailable")?;
    let mut count = 0u64;
    let mut bytes = 0u64;
    let entries = std::fs::read_dir(&temp)
        .map_err(|e| format!("Cannot inspect {}: {e}", temp.display()))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if let Ok(meta) = std::fs::symlink_metadata(&path) {
            if meta.file_type().is_file() {
                count += 1;
                bytes = bytes.saturating_add(meta.len());
            }
        }
    }
    Ok(format!(
        "Temporary folder: {}\nTop-level files: {count}\nPotentially reclaimable file size: {:.1} MB\n\nPreview only: nothing was deleted. Locked files, nested folders, caches and app-owned data are intentionally excluded.",
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
    run("tasklist.exe", &["/FO", "TABLE", "/NH"], Duration::from_secs(6)).map(bounded_output)
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
    run("powercfg.exe", &["/getactivescheme"], Duration::from_secs(5))
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
