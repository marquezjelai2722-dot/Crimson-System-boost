use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
    process::Command,
    time::UNIX_EPOCH,
};

/// Hashes a file incrementally so large files are not loaded into memory at once.
pub fn sha256_file(path: &Path) -> io::Result<[u8; 32]> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 16 * 1024];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hasher.finalize().into())
}

#[cfg(windows)]
fn defender_cli() -> Option<PathBuf> {
    let mut candidates: Vec<(std::time::SystemTime, PathBuf)> = Vec::new();

    if let Some(program_data) = std::env::var_os("ProgramData") {
        let platform_dir = PathBuf::from(program_data)
            .join("Microsoft")
            .join("Windows Defender")
            .join("Platform");

        if let Ok(entries) = std::fs::read_dir(platform_dir) {
            for entry in entries.flatten() {
                let candidate = entry.path().join("MpCmdRun.exe");
                if let Ok(metadata) = candidate.metadata() {
                    if metadata.is_file() {
                        let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
                        candidates.push((modified, candidate));
                    }
                }
            }
        }
    }

    candidates.sort_by_key(|(modified, _)| *modified);
    if let Some((_, newest)) = candidates.pop() {
        return Some(newest);
    }

    std::env::var_os("ProgramFiles")
        .map(|program_files| {
            PathBuf::from(program_files)
                .join("Windows Defender")
                .join("MpCmdRun.exe")
        })
        .filter(|path| path.is_file())
}

#[cfg(windows)]
pub fn run_defender_scan(target: &Path) -> String {
    let Some(cli) = defender_cli() else {
        return "Microsoft Defender command-line scanner was not found. Check that Microsoft Defender Antivirus is installed and enabled.".to_owned();
    };

    let mut child = match Command::new(cli)
        .args(["-Scan", "-ScanType", "3", "-File"])
        .arg(target)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => return format!("Could not start Microsoft Defender: {error}"),
    };

    // A stuck Defender process must not leave the scan UI permanently busy.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if std::time::Instant::now() < deadline => {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return "Microsoft Defender scan timed out after 30 seconds. Check Windows Security for scan status.".to_owned();
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return format!("Could not monitor Microsoft Defender scan: {error}");
            }
        }
    }

    match child.wait() {
        Ok(status) if status.success() => {
            format!("Defender custom scan finished (exit code {}). Check Windows Security → Protection history for detections.", status.code().unwrap_or(0))
        }
        Ok(status) => {
            format!("Defender scan did not complete successfully (exit code {:?}). Check Windows Security → Protection history. No firewall settings were changed.", status.code())
        }
        Err(error) => format!("Could not collect Microsoft Defender scan result: {error}"),
    }
}

#[cfg(not(windows))]
pub fn run_defender_scan(_target: &Path) -> String {
    "Microsoft Defender integration is available only on Windows.".to_owned()
}

#[cfg(test)]
mod tests {
    use super::sha256_file;
    use std::{fs, path::PathBuf};

    #[test]
    fn sha256_matches_known_vector() {
        let path = PathBuf::from(std::env::temp_dir())
            .join(format!("crimson-sha256-test-{}.tmp", std::process::id()));
        fs::write(&path, b"abc").expect("write test file");

        let actual = sha256_file(&path).expect("hash test file");
        let expected = [
            0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae,
            0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61,
            0xf2, 0x00, 0x15, 0xad,
        ];

        let _ = fs::remove_file(&path);
        assert_eq!(actual, expected);
    }
}
