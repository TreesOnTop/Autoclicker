use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

const SETTINGS_FORMAT_VERSION: u8 = 3;
const V2_HEADER_LEN: usize = 18;
const V3_HEADER_LEN: usize = 26;

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
        Self {
            name,
            action,
            enabled,
        }
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
    pub corner_stop_tl: u16,
    pub corner_stop_tr: u16,
    pub corner_stop_bl: u16,
    pub corner_stop_br: u16,
    pub edge_stop_top: u16,
    pub edge_stop_right: u16,
    pub edge_stop_bottom: u16,
    pub edge_stop_left: u16,
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
            corner_stop_tl: 0,
            corner_stop_tr: 0,
            corner_stop_bl: 0,
            corner_stop_br: 0,
            edge_stop_top: 0,
            edge_stop_right: 0,
            edge_stop_bottom: 0,
            edge_stop_left: 0,
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

fn parse_processes(process_data: &[u8]) -> Vec<ProcessEntry> {
    let mut processes = Vec::new();
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
    processes
}

fn write_processes(w: &mut Vec<u8>, settings: &AppSettings) {
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
}

pub fn serialize_custom(settings: &AppSettings) -> Vec<u8> {
    let mut w = Vec::new();
    w.push(SETTINGS_FORMAT_VERSION);

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
    flags |= ((settings.click_type_index as u8) & 0b11) << 3;
    if settings.filter_mode != 0 {
        flags |= 1 << 5;
    }
    w.push(flags);

    w.extend_from_slice(&settings.current_hotkey.to_le_bytes());
    w.extend_from_slice(&settings.interval_ms.to_le_bytes());

    for value in [
        settings.corner_stop_tl,
        settings.corner_stop_tr,
        settings.corner_stop_bl,
        settings.corner_stop_br,
        settings.edge_stop_top,
        settings.edge_stop_right,
        settings.edge_stop_bottom,
        settings.edge_stop_left,
    ] {
        w.extend_from_slice(&value.to_le_bytes());
    }

    write_processes(&mut w, settings);
    w
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(bytes[offset..offset + 2].try_into().ok()?))
}

fn deserialize_custom(bytes: &[u8]) -> Option<(AppSettings, u8)> {
    if bytes.is_empty() {
        return None;
    }

    let version = bytes[0];
    match version {
        1 => deserialize_v2(bytes).map(|s| (s, 1)),
        2 => deserialize_v2(bytes).map(|s| (s, 2)),
        3 => deserialize_v3(bytes).map(|s| (s, 3)),
        _ => None,
    }
}

fn deserialize_v2(bytes: &[u8]) -> Option<AppSettings> {
    if bytes.len() < V2_HEADER_LEN {
        return None;
    }

    let flags = bytes[1];
    let always_on_top = (flags & (1 << 0)) != 0;
    let minimize_to_tray = (flags & (1 << 1)) != 0;
    let pause_on_window_change = (flags & (1 << 2)) != 0;

    let current_hotkey = i32::from_le_bytes(bytes[2..6].try_into().ok()?);
    let interval_ms = i32::from_le_bytes(bytes[6..10].try_into().ok()?);
    let click_type_index = i32::from_le_bytes(bytes[10..14].try_into().ok()?);
    let filter_mode = i32::from_le_bytes(bytes[14..18].try_into().ok()?);

    let processes = if bytes.len() > V2_HEADER_LEN {
        parse_processes(&bytes[V2_HEADER_LEN..])
    } else {
        Vec::new()
    };

    Some(AppSettings {
        always_on_top,
        minimize_to_tray,
        pause_on_window_change,
        current_hotkey,
        interval_ms,
        click_type_index,
        filter_mode,
        corner_stop_tl: 0,
        corner_stop_tr: 0,
        corner_stop_bl: 0,
        corner_stop_br: 0,
        edge_stop_top: 0,
        edge_stop_right: 0,
        edge_stop_bottom: 0,
        edge_stop_left: 0,
        processes,
    })
}

fn deserialize_v3(bytes: &[u8]) -> Option<AppSettings> {
    if bytes.len() < V3_HEADER_LEN {
        return None;
    }

    let flags = bytes[1];
    let always_on_top = (flags & (1 << 0)) != 0;
    let minimize_to_tray = (flags & (1 << 1)) != 0;
    let pause_on_window_change = (flags & (1 << 2)) != 0;
    let click_type_index = ((flags >> 3) & 0b11) as i32;
    let filter_mode = if (flags & (1 << 5)) != 0 { 1 } else { 0 };

    let current_hotkey = i32::from_le_bytes(bytes[2..6].try_into().ok()?);
    let interval_ms = i32::from_le_bytes(bytes[6..10].try_into().ok()?);

    let corner_stop_tl = read_u16(bytes, 10)?;
    let corner_stop_tr = read_u16(bytes, 12)?;
    let corner_stop_bl = read_u16(bytes, 14)?;
    let corner_stop_br = read_u16(bytes, 16)?;
    let edge_stop_top = read_u16(bytes, 18)?;
    let edge_stop_right = read_u16(bytes, 20)?;
    let edge_stop_bottom = read_u16(bytes, 22)?;
    let edge_stop_left = read_u16(bytes, 24)?;

    let processes = if bytes.len() > V3_HEADER_LEN {
        parse_processes(&bytes[V3_HEADER_LEN..])
    } else {
        Vec::new()
    };

    Some(AppSettings {
        always_on_top,
        minimize_to_tray,
        pause_on_window_change,
        current_hotkey,
        interval_ms,
        click_type_index,
        filter_mode,
        corner_stop_tl,
        corner_stop_tr,
        corner_stop_bl,
        corner_stop_br,
        edge_stop_top,
        edge_stop_right,
        edge_stop_bottom,
        edge_stop_left,
        processes,
    })
}

pub fn load_settings() -> AppSettings {
    let path = get_settings_path();
    if path.exists() {
        if let Ok(mut file) = File::open(&path) {
            let mut contents = Vec::new();
            if file.read_to_end(&mut contents).is_ok() {
                if let Some((settings, version)) = deserialize_custom(&contents) {
                    if version < SETTINGS_FORMAT_VERSION {
                        save_settings(&settings);
                    }
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
