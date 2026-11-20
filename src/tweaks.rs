//! Small, explicitly allow-listed Windows tweaks with snapshots and rollback.
//! No shell text or registry paths are accepted from the UI.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preset {
    OneClick,
    PlatinumPlus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TweakId {
    GameCaptureOff,
    GameModeOn,
    TransparencyOff,
    VisualEffectsPerformance,
    DisableConsumerFeatures,
    DisableAppSuggestions,
    DisableWindowsTips,
}

impl TweakId {
    pub const ALL: [Self; 7] = [
        Self::GameCaptureOff,
        Self::GameModeOn,
        Self::TransparencyOff,
        Self::VisualEffectsPerformance,
        Self::DisableConsumerFeatures,
        Self::DisableAppSuggestions,
        Self::DisableWindowsTips,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::GameCaptureOff => "Disable background game recording",
            Self::GameModeOn => "Enable Windows Game Mode",
            Self::TransparencyOff => "Disable transparency effects",
            Self::VisualEffectsPerformance => "Prioritize visual performance",
            Self::DisableConsumerFeatures => "Disable Windows consumer experiences",
            Self::DisableAppSuggestions => "Disable Start menu app suggestions",
            Self::DisableWindowsTips => "Disable Windows tips and suggestions",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::GameCaptureOff => {
                "Turns off Xbox Game Bar background capture. This also stops Windows from keeping automatic gameplay clips."
            }
            Self::GameModeOn => {
                "Enables Windows Game Mode so Windows can prioritize the active game."
            }
            Self::TransparencyOff => {
                "Turns off translucent Windows surfaces. This is a small graphics/UI change, not an FPS guarantee."
            }
            Self::VisualEffectsPerformance => {
                "Selects Windows visual-effects performance mode. Animations and visual polish may be reduced."
            }
            Self::DisableConsumerFeatures => {
                "Turns off Windows consumer-experience suggestions for the current user. It does not disable Windows Update or security features."
            }
            Self::DisableAppSuggestions => {
                "Turns off suggested apps in the Start menu for the current user."
            }
            Self::DisableWindowsTips => {
                "Turns off Windows tips and suggested-content notifications for the current user."
            }
        }
    }
}

#[cfg(windows)]
mod windows {
    use super::{Preset, TweakId};
    use std::{
        fs, io,
        path::{Path, PathBuf},
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };
    use winreg::{
        enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_SET_VALUE, REG_DWORD},
        RegKey, RegValue,
    };

    const SNAPSHOT_HEADER: &str = "CRIMSON-SYSTEM-BOOST-SNAPSHOT-V1";

    #[derive(Clone, Copy)]
    struct Target {
        key: &'static str,
        name: &'static str,
        desired: u32,
    }

    const GAME_CAPTURE: [Target; 2] = [
        Target {
            key: "Software\\Microsoft\\Windows\\CurrentVersion\\GameDVR",
            name: "AppCaptureEnabled",
            desired: 0,
        },
        Target {
            key: "System\\GameConfigStore",
            name: "GameDVR_Enabled",
            desired: 0,
        },
    ];

    const GAME_MODE: [Target; 2] = [
        Target {
            key: "Software\\Microsoft\\GameBar",
            name: "AllowAutoGameMode",
            desired: 1,
        },
        Target {
            key: "Software\\Microsoft\\GameBar",
            name: "AutoGameModeEnabled",
            desired: 1,
        },
    ];

    const TRANSPARENCY: [Target; 1] = [Target {
        key: "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
        name: "EnableTransparency",
        desired: 0,
    }];

    const VISUAL_EFFECTS: [Target; 1] = [Target {
        key: "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\VisualEffects",
        name: "VisualFXSetting",
        desired: 2,
    }];

    const CONSUMER_FEATURES: [Target; 1] = [Target {
        key: "Software\\Policies\\Microsoft\\Windows\\CloudContent",
        name: "DisableWindowsConsumerFeatures",
        desired: 1,
    }];

    const APP_SUGGESTIONS: [Target; 1] = [Target {
        key: "Software\\Microsoft\\Windows\\CurrentVersion\\ContentDeliveryManager",
        name: "SystemPaneSuggestionsEnabled",
        desired: 0,
    }];

    const WINDOWS_TIPS: [Target; 1] = [Target {
        key: "Software\\Microsoft\\Windows\\CurrentVersion\\ContentDeliveryManager",
        name: "SubscribedContent-338389Enabled",
        desired: 0,
    }];

    fn targets(id: TweakId) -> &'static [Target] {
        match id {
            TweakId::GameCaptureOff => &GAME_CAPTURE,
            TweakId::GameModeOn => &GAME_MODE,
            TweakId::TransparencyOff => &TRANSPARENCY,
            TweakId::VisualEffectsPerformance => &VISUAL_EFFECTS,
            TweakId::DisableConsumerFeatures => &CONSUMER_FEATURES,
            TweakId::DisableAppSuggestions => &APP_SUGGESTIONS,
            TweakId::DisableWindowsTips => &WINDOWS_TIPS,
        }
    }

    fn id_name(id: TweakId) -> &'static str {
        match id {
            TweakId::GameCaptureOff => "game_capture_off",
            TweakId::GameModeOn => "game_mode_on",
            TweakId::TransparencyOff => "transparency_off",
            TweakId::VisualEffectsPerformance => "visual_effects_performance",
            TweakId::DisableConsumerFeatures => "disable_consumer_features",
            TweakId::DisableAppSuggestions => "disable_app_suggestions",
            TweakId::DisableWindowsTips => "disable_windows_tips",
        }
    }

    fn parse_id(value: &str) -> Option<TweakId> {
        TweakId::ALL.into_iter().find(|id| id_name(*id) == value)
    }

    fn timestamp() -> Result<u128, String> {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis())
            .map_err(|error| format!("System clock is invalid: {error}"))
    }

    fn backup_dir() -> Result<PathBuf, String> {
        let local_app_data = std::env::var_os("LOCALAPPDATA").ok_or_else(|| {
            "LOCALAPPDATA is unavailable; cannot create a safe backup.".to_owned()
        })?;
        let directory = PathBuf::from(local_app_data)
            .join("Crimson System Boost")
            .join("Backups");
        fs::create_dir_all(&directory)
            .map_err(|error| format!("Could not create the backup directory: {error}"))?;
        Ok(directory)
    }

    fn verify_supported_windows() -> Result<(), String> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let key = hklm
            .open_subkey_with_flags("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion", KEY_READ)
            .map_err(|error| format!("Could not identify this Windows installation: {error}"))?;
        let build: String = key
            .get_value("CurrentBuildNumber")
            .map_err(|error| format!("Could not read the Windows build number: {error}"))?;
        let build_number = build
            .parse::<u32>()
            .map_err(|_| format!("Unrecognized Windows build number: {build}"))?;
        if build_number < 10_240 {
            return Err("Crimson tweaks support Windows 10 and Windows 11 only.".to_owned());
        }
        Ok(())
    }

    fn powershell_encoded(script: &str) -> String {
        let utf16: Vec<u8> = script.encode_utf16().flat_map(u16::to_le_bytes).collect();
        base64_encode(&utf16)
    }

    fn base64_encode(bytes: &[u8]) -> String {
        const TABLE: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut output = String::with_capacity(bytes.len().div_ceil(3) * 4);
        for chunk in bytes.chunks(3) {
            let a = chunk[0];
            let b = *chunk.get(1).unwrap_or(&0);
            let c = *chunk.get(2).unwrap_or(&0);
            output.push(TABLE[(a >> 2) as usize] as char);
            output.push(TABLE[(((a & 0x03) << 4) | (b >> 4)) as usize] as char);
            output.push(if chunk.len() > 1 {
                TABLE[(((b & 0x0f) << 2) | (c >> 6)) as usize] as char
            } else {
                '='
            });
            output.push(if chunk.len() > 2 {
                TABLE[(c & 0x3f) as usize] as char
            } else {
                '='
            });
        }
        output
    }

    /// Windows requires elevation to create a System Restore point. The only
    /// elevated command is a fixed Checkpoint-Computer invocation.
    fn create_restore_point() -> Result<(), String> {
        let elevated_script = "try { Checkpoint-Computer -Description 'Crimson System Boost - before optimization' -RestorePointType 'MODIFY_SETTINGS' -ErrorAction Stop; exit 0 } catch { Write-Error $_; exit 1 }";
        let encoded_child = powershell_encoded(elevated_script);
        let parent_script = format!(
            "try {{ $p = Start-Process -FilePath 'powershell.exe' -Verb RunAs -Wait -PassThru -ArgumentList '-NoProfile -NonInteractive -EncodedCommand {encoded_child}'; exit $p.ExitCode }} catch {{ Write-Error $_; exit 2 }}"
        );
        let output = Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &parent_script,
            ])
            .output()
            .map_err(|error| format!("Could not start the restore-point helper: {error}"))?;
        if !output.status.success() {
            let detail = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "Windows did not confirm a restore point. No tweaks were applied. Enable System Protection and try again. {}",
                detail.trim()
            ));
        }
        Ok(())
    }

    fn read_original(target: Target) -> Result<Option<u32>, String> {
        let root = RegKey::predef(HKEY_CURRENT_USER);
        let key = match root.open_subkey_with_flags(target.key, KEY_READ) {
            Ok(key) => key,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(format!("Could not read HKCU\\{}: {error}", target.key)),
        };
        match key.get_raw_value(target.name) {
            Ok(value) => {
                if value.vtype != REG_DWORD || value.bytes.len() != 4 {
                    return Err(format!(
                        "HKCU\\{}\\{} has an unexpected registry type. It was left untouched.",
                        target.key, target.name
                    ));
                }
                Ok(Some(u32::from_le_bytes([
                    value.bytes[0],
                    value.bytes[1],
                    value.bytes[2],
                    value.bytes[3],
                ])))
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(format!(
                "Could not read HKCU\\{}\\{}: {error}",
                target.key, target.name
            )),
        }
    }

    fn write_dword(target: Target, value: u32) -> Result<(), String> {
        let root = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = root
            .create_subkey(target.key)
            .map_err(|error| format!("Could not open HKCU\\{}: {error}", target.key))?;
        key.set_raw_value(
            target.name,
            &RegValue {
                vtype: REG_DWORD,
                bytes: value.to_le_bytes().to_vec(),
            },
        )
        .map_err(|error| {
            format!(
                "Could not update HKCU\\{}\\{}: {error}",
                target.key, target.name
            )
        })
    }

    fn delete_value(target: Target) -> Result<(), String> {
        let root = RegKey::predef(HKEY_CURRENT_USER);
        let key = match root.open_subkey_with_flags(target.key, KEY_SET_VALUE) {
            Ok(key) => key,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(format!("Could not open HKCU\\{}: {error}", target.key)),
        };
        match key.delete_value(target.name) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(format!(
                "Could not remove HKCU\\{}\\{}: {error}",
                target.key, target.name
            )),
        }
    }

    fn snapshot_text(ids: &[TweakId]) -> Result<String, String> {
        let mut text = format!("{SNAPSHOT_HEADER}\n");
        for id in ids {
            for (index, target) in targets(*id).iter().enumerate() {
                match read_original(*target)? {
                    Some(value) => {
                        text.push_str(&format!("{}\t{}\tD\t{}\n", id_name(*id), index, value));
                    }
                    None => text.push_str(&format!("{}\t{}\tM\n", id_name(*id), index)),
                }
            }
        }
        Ok(text)
    }

    fn restore_snapshot(snapshot: &str) -> Result<(), String> {
        let mut lines = snapshot.lines();
        if lines.next() != Some(SNAPSHOT_HEADER) {
            return Err("Backup format is invalid. Nothing was restored.".to_owned());
        }

        let mut operations = Vec::new();
        for line in lines {
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() < 3 {
                return Err("Backup contains an invalid entry. Nothing was restored.".to_owned());
            }
            let id = parse_id(fields[0])
                .ok_or_else(|| "Backup contains an unknown tweak identifier.".to_owned())?;
            let index = fields[1]
                .parse::<usize>()
                .map_err(|_| "Backup contains an invalid target index.".to_owned())?;
            let target = *targets(id)
                .get(index)
                .ok_or_else(|| "Backup contains an out-of-range target index.".to_owned())?;
            match fields[2] {
                "M" if fields.len() == 3 => operations.push((target, None)),
                "D" if fields.len() == 4 => {
                    let value = fields[3]
                        .parse::<u32>()
                        .map_err(|_| "Backup contains an invalid DWORD.".to_owned())?;
                    operations.push((target, Some(value)));
                }
                _ => return Err("Backup contains an unsupported value type.".to_owned()),
            }
        }

        for (target, value) in operations {
            match value {
                Some(value) => write_dword(target, value)?,
                None => delete_value(target)?,
            }
            let actual = read_original(target)?;
            if actual != value {
                return Err(format!(
                    "Rollback verification failed for HKCU\\{}\\{}.",
                    target.key, target.name
                ));
            }
        }
        Ok(())
    }

    fn selected_for_preset(preset: Preset) -> Vec<TweakId> {
        match preset {
            Preset::OneClick => vec![
                TweakId::GameCaptureOff,
                TweakId::GameModeOn,
                TweakId::TransparencyOff,
            ],
            Preset::PlatinumPlus => TweakId::ALL.to_vec(),
        }
    }

    pub fn apply_preset(preset: Preset) -> Result<String, String> {
        apply_selected(&selected_for_preset(preset))
    }

    pub fn apply_selected(ids: &[TweakId]) -> Result<String, String> {
        if ids.is_empty() {
            return Err("Select at least one tweak before applying.".to_owned());
        }
        verify_supported_windows()?;
        create_restore_point()?;

        let snapshot = snapshot_text(ids)?;
        let directory = backup_dir()?;
        let path = directory.join(format!("apply-{}.snapshot", timestamp()?));
        fs::write(&path, snapshot.as_bytes())
            .map_err(|error| format!("Could not save the settings snapshot: {error}"))?;

        let mut changed = 0usize;
        for id in ids {
            for target in targets(*id) {
                let apply_result = write_dword(*target, target.desired).and_then(|()| {
                    match read_original(*target)? {
                        Some(actual) if actual == target.desired => Ok(()),
                        actual => Err(format!(
                            "Verification failed for HKCU\\{}\\{} (expected {}, found {:?}).",
                            target.key, target.name, target.desired, actual
                        )),
                    }
                });
                if let Err(error) = apply_result {
                    let rollback = fs::read_to_string(&path)
                        .map_err(|read_error| format!("{error}; backup read failed: {read_error}"))
                        .and_then(|saved| restore_snapshot(&saved));
                    return match rollback {
                        Ok(()) => Err(format!("{error} Previous values were restored.")),
                        Err(rollback_error) => Err(format!(
                            "{error} Automatic rollback also failed: {rollback_error}. Backup: {}",
                            path.display()
                        )),
                    };
                }
                changed += 1;
            }
        }

        Ok(format!(
            "Applied {changed} registry setting(s). A verified System Restore point and settings snapshot were created. Restart affected apps or sign out for every change to take effect. Backup: {}",
            path.display()
        ))
    }

    pub fn undo_latest() -> Result<String, String> {
        verify_supported_windows()?;
        let directory = backup_dir()?;
        let mut backups: Vec<PathBuf> = fs::read_dir(&directory)
            .map_err(|error| format!("Could not list settings backups: {error}"))?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                path.extension()
                    .is_some_and(|extension| extension == "snapshot")
                    && path
                        .file_name()
                        .is_some_and(|name| name.to_string_lossy().starts_with("apply-"))
            })
            .collect();
        backups.sort_by_key(|path| {
            fs::metadata(path)
                .and_then(|metadata| metadata.modified())
                .unwrap_or(UNIX_EPOCH)
        });
        let path = backups
            .pop()
            .ok_or_else(|| "No Crimson optimization backup was found.".to_owned())?;

        create_restore_point()?;
        let snapshot = fs::read_to_string(&path)
            .map_err(|error| format!("Could not read the settings backup: {error}"))?;
        let ids = snapshot
            .lines()
            .skip(1)
            .filter_map(|line| line.split('\t').next())
            .filter_map(parse_id)
            .collect::<Vec<_>>();
        let current = snapshot_text(&ids)?;
        let undo_path = directory.join(format!("undo-{}.snapshot", timestamp()?));
        fs::write(&undo_path, current.as_bytes())
            .map_err(|error| format!("Could not save the pre-undo snapshot: {error}"))?;

        restore_snapshot(&snapshot)?;
        Ok(format!(
            "Restored the previous values from {}. A pre-undo snapshot was also saved. Some changes may require signing out or restarting Windows.",
            path.display()
        ))
    }

    #[allow(dead_code)]
    fn _path_is_inside_backup(path: &Path, directory: &Path) -> bool {
        path.starts_with(directory)
    }
}

#[cfg(windows)]
pub use windows::{apply_preset, apply_selected, undo_latest};

#[cfg(not(windows))]
pub fn apply_preset(_preset: Preset) -> Result<String, String> {
    Err("Windows optimization is available only on Windows 10 and Windows 11.".to_owned())
}

#[cfg(not(windows))]
pub fn apply_selected(_ids: &[TweakId]) -> Result<String, String> {
    Err("Windows optimization is available only on Windows 10 and Windows 11.".to_owned())
}

#[cfg(not(windows))]
pub fn undo_latest() -> Result<String, String> {
    Err("Windows optimization is available only on Windows 10 and Windows 11.".to_owned())
}
