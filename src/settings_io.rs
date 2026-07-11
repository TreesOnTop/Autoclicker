use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessEntry {
    pub name: String,
    pub action: i32,
    #[serde(skip, default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

impl ProcessEntry {
    pub fn new(name: String, action: i32, enabled: bool) -> Self {
        Self { name, action, enabled }
    }
}

#[derive(Clone, Debug)]
pub struct AppSettings {
    pub always_on_top: bool,
    pub minimize_to_tray: bool,
    pub pause_on_window_change: bool,
    pub current_hotkey: i32,
    pub interval_ms: i32,
    pub click_type_index: i32,
    pub filter_mode: i32,
    pub processes: Vec<ProcessEntry>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            always_on_top: false,
            minimize_to_tray: false,
            pause_on_window_change: false,
            current_hotkey: 0xFFC7,
            interval_ms: 100,
            click_type_index: 0,
            filter_mode: 0,
            processes: vec![],
        }
    }
}

fn get_settings_path() -> PathBuf {
    if let Ok(app_data) = std::env::var("LOCALAPPDATA") {
        let mut path = PathBuf::from(app_data);
        path.push("TreeAutoClicker");
        let _ = std::fs::create_dir_all(&path);
        path.push("settings.bin");
        path
    } else {
        PathBuf::from("settings.bin")
    }
}

pub fn serialize_custom(settings: &AppSettings) -> Vec<u8> {
    let mut w = Vec::new();
    w.push(2u8);

    let mut flags = 0u8;
    if settings.always_on_top {
        flags |= 1 << 0;
    }
    if settings.minimize_to_tray {
        flags |= 1 << 1;
    }
    if settings.pause_on_window_change {
        flags |= 1 << 2;
    }
    w.push(flags);

    w.extend_from_slice(&settings.current_hotkey.to_le_bytes());
    w.extend_from_slice(&settings.interval_ms.to_le_bytes());
    w.extend_from_slice(&settings.click_type_index.to_le_bytes());
    w.extend_from_slice(&settings.filter_mode.to_le_bytes());

    let joined = settings
        .processes
        .iter()
        .filter(|e| e.enabled)
        .map(|e| format!("{}\x01{}", e.name, e.action))
        .collect::<Vec<_>>()
        .join("\0");

    if !joined.is_empty() {
        w.extend_from_slice(joined.as_bytes());
    }

    w
}

pub fn deserialize_custom(bytes: &[u8]) -> Option<AppSettings> {
    if bytes.len() < 18 {
        return None;
    }

    let version = bytes[0];

    let flags = bytes[1];
    let always_on_top = (flags & (1 << 0)) != 0;
    let minimize_to_tray = (flags & (1 << 1)) != 0;
    let pause_on_window_change = (flags & (1 << 2)) != 0;

    let current_hotkey = i32::from_le_bytes(bytes[2..6].try_into().ok()?);
    let interval_ms = i32::from_le_bytes(bytes[6..10].try_into().ok()?);
    let click_type_index = i32::from_le_bytes(bytes[10..14].try_into().ok()?);
    let filter_mode = i32::from_le_bytes(bytes[14..18].try_into().ok()?);

    let process_start_idx = match version {
        1 => {
            if bytes.len() >= 22 { 22 } else { 18 }
        }
        _ => 18, // version 2+
    };

    let mut processes = Vec::new();
    if bytes.len() > process_start_idx {
        let process_data = &bytes[process_start_idx..];
        if let Ok(string_data) = std::str::from_utf8(process_data) {
            for entry_str in string_data.split('\0').filter(|s| !s.is_empty()) {
                if let Some(sep_idx) = entry_str.find('\x01') {
                    let name = entry_str[..sep_idx].to_string();
                    let action = entry_str[sep_idx + 1..].parse::<i32>().unwrap_or(1);
                    processes.push(ProcessEntry::new(name, action, true));
                } else {
                    processes.push(ProcessEntry::new(entry_str.to_string(), 1, true));
                }
            }
        }
    }

    Some(AppSettings {
        always_on_top,
        minimize_to_tray,
        pause_on_window_change,
        current_hotkey,
        interval_ms,
        click_type_index,
        filter_mode,
        processes,
    })
}

pub fn load_settings() -> AppSettings {
    let path = get_settings_path();
    if path.exists() {
        if let Ok(mut file) = File::open(&path) {
            let mut contents = Vec::new();
            if file.read_to_end(&mut contents).is_ok() {
                if let Some(settings) = deserialize_custom(&contents) {
                    return settings;
                }
            }
        }
    }
    AppSettings::default()
}

pub fn save_settings(settings: &AppSettings) {
    let path = get_settings_path();
    let bytes = serialize_custom(settings);
    if let Ok(mut file) = File::create(&path) {
        let _ = file.write_all(&bytes);
    }
}
