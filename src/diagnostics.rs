//! Read-only system diagnostics. These commands inspect state; they never modify it.
use std::{path::PathBuf, process::Command, time::Duration};

fn run(program: &str, args: &[&str], timeout: Duration) -> Result<String, String> {
    let mut child = Command::new(program).args(args).stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped()).spawn()
        .map_err(|e| format!("Could not start {program}: {e}"))?;
    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                let output = child.wait_with_output().map_err(|e| e.to_string())?;
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
                let text = if stdout.is_empty() { stderr } else { stdout };
                if text.is_empty() { return Err(format!("{program} returned no diagnostic output")); }
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
        let result = Command::new("reg.exe").args(["query", key]).output()
            .map_err(|e| format!("Could not query startup entries: {e}"))?;
        let output = String::from_utf8_lossy(&result.stdout).trim().to_owned();
        if result.status.success() && !output.is_empty() { sections.push(output); }
        else { sections.push(format!("{key}\n  (no entries or inaccessible)")); }
    }
    Ok(sections.join("\n\n"))
}

#[cfg(not(windows))]
pub fn startup_inventory() -> Result<String, String> {
    Err("Startup inventory is currently supported on Windows only.".into())
}

/// Measures candidate temporary files. It deliberately does not delete anything.
pub fn cleanup_preview() -> Result<String, String> {
    let temp = std::env::var_os("TEMP").or_else(|| std::env::var_os("TMP"))
        .map(PathBuf::from).ok_or("Temporary directory is unavailable")?;
    let mut count = 0u64;
    let mut bytes = 0u64;
    let entries = std::fs::read_dir(&temp).map_err(|e| format!("Cannot inspect {}: {e}", temp.display()))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if let Ok(meta) = std::fs::symlink_metadata(&path) {
            if meta.file_type().is_file() {
                count += 1;
                bytes = bytes.saturating_add(meta.len());
            }
        }
    }
    Ok(format!("Temporary folder: {}\nTop-level files: {count}\nPotentially reclaimable file size: {:.1} MB\n\nPreview only: nothing was deleted. Locked files, nested folders, caches and app-owned data are intentionally excluded.",
        temp.display(), bytes as f64 / 1_048_576.0))
}

/// Performs bounded DNS and ping diagnostics against a caller-selected host.
pub fn network_test(host: &str) -> Result<String, String> {
    let host = host.trim();
    if host.is_empty() || host.len() > 253 || !host.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-')) {
        return Err("Enter a valid hostname (letters, numbers, dots and hyphens only).".into());
    }
    #[cfg(windows)]
    let (dns_program, dns_args, ping_args): (&str, Vec<&str>, Vec<&str>) =
        ("nslookup", vec![host], vec!["-n", "4", host]);
    #[cfg(not(windows))]
    let (dns_program, dns_args, ping_args): (&str, Vec<&str>, Vec<&str>) =
        ("nslookup", vec![host], vec!["-c", "4", host]);
    let dns = run(dns_program, &dns_args, Duration::from_secs(8))
        .unwrap_or_else(|e| format!("DNS lookup failed: {e}"));
    let ping = run("ping", &ping_args, Duration::from_secs(10))
        .unwrap_or_else(|e| format!("Ping failed: {e}"));
    Ok(format!("DNS lookup for {host}:\n{dns}\n\nPing test:\n{ping}\n\nDiagnostics only: no adapter, DNS or firewall settings were changed."))
}

#[cfg(test)]
mod tests {
    #[test]
    fn rejects_invalid_network_hosts() {
        assert!(super::network_test("bad host").is_err());
        assert!(super::network_test("").is_err());
    }
}
