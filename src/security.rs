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
                    let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
                    candidates.push((modified, candidate));
                }
            }
        }
    }

    candidates.sort_by_key(|(modified, _)| *modified);
    if let Some((_, newest)) = candidates.pop() {
        return Some(newest);
    }

    std::env::var_os("ProgramFiles").map(|program_files| {
        PathBuf::from(program_files)
            .join("Windows Defender")
            .join("MpCmdRun.exe")
    }).filter(|path| path.is_file())
}

#[cfg(windows)]
pub fn run_defender_scan(target: &Path) -> String {
    let Some(cli) = defender_cli() else {
        return "Microsoft Defender command-line scanner was not found. Check that Microsoft Defender Antivirus is installed and enabled.".to_owned();
    };

    match Command::new(cli)
        .args(["-Scan", "-ScanType", "3", "-File"])
        .arg(target)
        .output()
    {
        Ok(output) if output.status.success() => {
            format!("Defender custom scan finished successfully (exit code {}). Check Windows Security → Protection history for any detections.", output.status.code().unwrap_or(0))
        }
        Ok(output) => {
            format!("Defender scan did not complete successfully (exit code {:?}). Check Windows Security → Protection history. No firewall settings were changed.", output.status.code())
        }
        Err(error) => format!("Could not start Microsoft Defender: {error}"),
    }
}

#[cfg(not(windows))]
pub fn run_defender_scan(_target: &Path) -> String {
    "Microsoft Defender integration is available only on Windows.".to_owned()
}
