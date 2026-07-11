use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use crate::speed::{
    DEFAULT_RATE_COUNT, DEFAULT_RATE_UNIT, DelayParts, MAX_DELAY_HOURS, SPEED_MODE_DELAY,
    clamp_rate_count, compute_interval_ms, normalize_rate_unit, normalize_speed_mode,
};

const SETTINGS_FORMAT_VERSION: u8 = 4;
const V2_HEADER_LEN: usize = 18;
const V3_HEADER_LEN: usize = 26;
const V4_HEADER_LEN: usize = 37;

#[derive(Clone, Debug)]
pub struct ProcessEntry {
    pub name: String,
    pub action: i32,
    pub enabled: bool,
    normalized_name: String,
}
impl ProcessEntry {
    pub fn new(name: String, action: i32, enabled: bool) -> Self {
        let normalized_name = name.to_lowercase();
        Self {
            name,
            action,
            enabled,
            normalized_name,
        }
    }

    pub fn matches_normalized(&self, title: &str, process_name: &str) -> bool {
        !self.normalized_name.is_empty()
            && (title.contains(&self.normalized_name)
                || process_name.contains(&self.normalized_name))
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
    pub speed_mode: i32,
    pub delay_h: u16,
    pub delay_m: u8,
    pub delay_s: u8,
    pub delay_ms: u16,
    pub rate_count: i32,
    pub rate_unit: i32,
    pub processes: Vec<ProcessEntry>,
}

impl Default for AppSettings {
    fn default() -> Self {
        let delay_ms = 100u16;
        let interval_ms = compute_interval_ms(
            SPEED_MODE_DELAY,
            0,
            0,
            0,
            delay_ms,
            DEFAULT_RATE_COUNT,
            DEFAULT_RATE_UNIT,
        );
        Self {
            always_on_top: false,
            minimize_to_tray: false,
            pause_on_window_change: false,
            current_hotkey: 0xFFC7,
            interval_ms,
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
            speed_mode: SPEED_MODE_DELAY,
            delay_h: 0,
            delay_m: 0,
            delay_s: 0,
            delay_ms,
            rate_count: DEFAULT_RATE_COUNT,
            rate_unit: DEFAULT_RATE_UNIT,
            processes: vec![],
        }
    }
}

fn migrate_interval_to_delay(settings: &mut AppSettings) {
    let parts = DelayParts::from_interval_ms(settings.interval_ms);
    settings.speed_mode = SPEED_MODE_DELAY;
    settings.delay_h = parts.hours;
    settings.delay_m = parts.minutes;
    settings.delay_s = parts.seconds;
    settings.delay_ms = parts.milliseconds;
    settings.rate_count = DEFAULT_RATE_COUNT;
    settings.rate_unit = DEFAULT_RATE_UNIT;
    settings.interval_ms = compute_interval_ms(
        settings.speed_mode,
        settings.delay_h,
        settings.delay_m,
        settings.delay_s,
        settings.delay_ms,
        settings.rate_count,
        settings.rate_unit,
    );
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
    flags |= (settings.click_type_index.clamp(0, 2) as u8) << 3;
    if settings.filter_mode != 0 {
        flags |= 1 << 5;
    }
    flags |= (normalize_speed_mode(settings.speed_mode) as u8) << 6;
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

    w.extend_from_slice(&settings.delay_h.min(MAX_DELAY_HOURS).to_le_bytes());
    w.push(settings.delay_m.min(59));
    w.push(settings.delay_s.min(59));
    w.extend_from_slice(&settings.delay_ms.min(999).to_le_bytes());
    let rate_unit = normalize_rate_unit(settings.rate_unit);
    w.extend_from_slice(&clamp_rate_count(settings.rate_count, rate_unit).to_le_bytes());
    w.push(rate_unit as u8);

    write_processes(&mut w, settings);
    w
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(
        bytes[offset..offset + 2].try_into().ok()?,
    ))
}

fn decode_flags(flags: u8) -> (bool, bool, bool, i32, i32, i32) {
    (
        (flags & (1 << 0)) != 0,
        (flags & (1 << 1)) != 0,
        (flags & (1 << 2)) != 0,
        ((flags >> 3) & 0b11).min(2) as i32,
        i32::from((flags & (1 << 5)) != 0),
        normalize_speed_mode((flags >> 6) as i32),
    )
}

fn read_stops(bytes: &[u8]) -> Option<[u16; 8]> {
    Some([
        read_u16(bytes, 10)?,
        read_u16(bytes, 12)?,
        read_u16(bytes, 14)?,
        read_u16(bytes, 16)?,
        read_u16(bytes, 18)?,
        read_u16(bytes, 20)?,
        read_u16(bytes, 22)?,
        read_u16(bytes, 24)?,
    ])
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
        4 => deserialize_v4(bytes).map(|s| (s, 4)),
        _ => None,
    }
}

fn deserialize_v2(bytes: &[u8]) -> Option<AppSettings> {
    if bytes.len() < V2_HEADER_LEN {
        return None;
    }

    let (always_on_top, minimize_to_tray, pause_on_window_change, _, _, _) = decode_flags(bytes[1]);

    let current_hotkey = i32::from_le_bytes(bytes[2..6].try_into().ok()?);
    let interval_ms = i32::from_le_bytes(bytes[6..10].try_into().ok()?);
    let click_type_index = i32::from_le_bytes(bytes[10..14].try_into().ok()?).clamp(0, 2);
    let filter_mode = i32::from(i32::from_le_bytes(bytes[14..18].try_into().ok()?) != 0);

    let processes = if bytes.len() > V2_HEADER_LEN {
        parse_processes(&bytes[V2_HEADER_LEN..])
    } else {
        Vec::new()
    };

    let mut settings = AppSettings {
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
        speed_mode: SPEED_MODE_DELAY,
        delay_h: 0,
        delay_m: 0,
        delay_s: 0,
        delay_ms: 0,
        rate_count: DEFAULT_RATE_COUNT,
        rate_unit: DEFAULT_RATE_UNIT,
        processes,
    };
    migrate_interval_to_delay(&mut settings);
    Some(settings)
}

fn deserialize_v3(bytes: &[u8]) -> Option<AppSettings> {
    if bytes.len() < V3_HEADER_LEN {
        return None;
    }

    let (always_on_top, minimize_to_tray, pause_on_window_change, click_type_index, filter_mode, _) =
        decode_flags(bytes[1]);

    let current_hotkey = i32::from_le_bytes(bytes[2..6].try_into().ok()?);
    let interval_ms = i32::from_le_bytes(bytes[6..10].try_into().ok()?);

    let [
        corner_stop_tl,
        corner_stop_tr,
        corner_stop_bl,
        corner_stop_br,
        edge_stop_top,
        edge_stop_right,
        edge_stop_bottom,
        edge_stop_left,
    ] = read_stops(bytes)?;

    let processes = if bytes.len() > V3_HEADER_LEN {
        parse_processes(&bytes[V3_HEADER_LEN..])
    } else {
        Vec::new()
    };

    let mut settings = AppSettings {
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
        speed_mode: SPEED_MODE_DELAY,
        delay_h: 0,
        delay_m: 0,
        delay_s: 0,
        delay_ms: 0,
        rate_count: DEFAULT_RATE_COUNT,
        rate_unit: DEFAULT_RATE_UNIT,
        processes,
    };
    migrate_interval_to_delay(&mut settings);
    Some(settings)
}

fn deserialize_v4(bytes: &[u8]) -> Option<AppSettings> {
    if bytes.len() < V4_HEADER_LEN {
        return None;
    }

    let (
        always_on_top,
        minimize_to_tray,
        pause_on_window_change,
        click_type_index,
        filter_mode,
        speed_mode,
    ) = decode_flags(bytes[1]);

    let current_hotkey = i32::from_le_bytes(bytes[2..6].try_into().ok()?);
    let _stored_interval_ms = i32::from_le_bytes(bytes[6..10].try_into().ok()?);

    let [
        corner_stop_tl,
        corner_stop_tr,
        corner_stop_bl,
        corner_stop_br,
        edge_stop_top,
        edge_stop_right,
        edge_stop_bottom,
        edge_stop_left,
    ] = read_stops(bytes)?;

    let delay_h = read_u16(bytes, 26)?.min(MAX_DELAY_HOURS);
    let delay_m = bytes[28].min(59);
    let delay_s = bytes[29].min(59);
    let delay_ms = read_u16(bytes, 30)?.min(999);
    let rate_unit = normalize_rate_unit(bytes[36] as i32);
    let rate_count = clamp_rate_count(
        i32::from_le_bytes(bytes[32..36].try_into().ok()?),
        rate_unit,
    );

    let processes = if bytes.len() > V4_HEADER_LEN {
        parse_processes(&bytes[V4_HEADER_LEN..])
    } else {
        Vec::new()
    };

    let interval_ms = compute_interval_ms(
        speed_mode, delay_h, delay_m, delay_s, delay_ms, rate_count, rate_unit,
    );

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
        speed_mode,
        delay_h,
        delay_m,
        delay_s,
        delay_ms,
        rate_count,
        rate_unit,
        processes,
    })
}

pub fn load_settings() -> AppSettings {
    let path = get_settings_path();
    if let Ok(mut file) = File::open(&path) {
        let mut contents = Vec::new();
        if file.read_to_end(&mut contents).is_ok()
            && let Some((settings, version)) = deserialize_custom(&contents)
        {
            if version < SETTINGS_FORMAT_VERSION {
                save_settings(&settings);
            }
            return settings;
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
