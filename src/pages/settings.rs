use crate::pages::widgets::{build_label, build_title, SegmentedControl};
use crate::ui::{col, CLR_GREEN, CLR_WIDGET};
use fltk::{app, button, enums::*, frame, group, prelude::*};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

pub struct SettingsHandles {
    pub group: group::Group,
    pub always_on_top_btn: button::CheckButton,
    pub minimize_to_tray_btn: button::CheckButton,
    pub pause_on_window_change_btn: button::CheckButton,
    pub current_hotkey: Arc<AtomicI32>,
    pub is_listening: Arc<std::sync::atomic::AtomicBool>,

    pub skip_next_hotkey: Arc<std::sync::atomic::AtomicBool>,
}

fn create_custom_toggle(
    x: i32,
    y: i32,
    default_val: bool,
    hidden_cb: &mut button::CheckButton,
) -> SegmentedControl {
    hidden_cb.set_value(default_val);

    let active_idx = if default_val { 1 } else { 0 };
    let mut seg = SegmentedControl::new(x, y, 80, 24, &["Off", "On"], active_idx);

    let mut cb_off = hidden_cb.clone();
    seg.set_callback(0, move |_| {
        cb_off.set_value(false);
        cb_off.do_callback();
    });

    let mut cb_on = hidden_cb.clone();
    seg.set_callback(1, move |_| {
        cb_on.set_value(true);
        cb_on.do_callback();
    });

    seg
}

pub fn build_settings_page(
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    initial_always_on_top: bool,
    initial_minimize_to_tray: bool,
    initial_pause_on_window_change: bool,
    initial_hotkey: i32,
    tx: std::sync::mpsc::Sender<()>,
) -> SettingsHandles {
    let mut settings_group = group::Group::default().with_size(w, h).with_pos(x, y);

    let mut card = group::Group::default()
        .with_size(w - 20, 230)
        .with_pos(x + 10, y + 10);
    card.set_frame(FrameType::RFlatBox);
    card.set_color(Color::from_rgb(20, 20, 20));

    let _title = build_title("Behavior", x + 25, y + 20, 200, 20, 14);

    let mut subtitle = build_label(
        "Change how the auto clicker runs.",
        x + 25,
        y + 40,
        250,
        20,
        12,
    );
    subtitle.set_label_color(Color::from_rgb(150, 150, 150));

    let mut sep = frame::Frame::default()
        .with_size(w - 40, 1)
        .with_pos(x + 20, y + 65);
    sep.set_frame(FrameType::FlatBox);
    sep.set_color(Color::from_rgb(35, 35, 35));

    let _aot_title = build_title("Always on Top", x + 25, y + 75, 150, 20, 13);

    let mut aot_subtitle =
        build_label("Keep the window above others.", x + 25, y + 93, 200, 20, 11);
    aot_subtitle.set_label_color(Color::from_rgb(150, 150, 150));

    let mut always_on_top_btn = button::CheckButton::default()
        .with_size(0, 0)
        .with_pos(0, 0);
    always_on_top_btn.hide();
    let _aot_seg = create_custom_toggle(
        x + w - 110,
        y + 78,
        initial_always_on_top,
        &mut always_on_top_btn,
    );

    let _mtt_title = build_title("Minimize to Tray", x + 25, y + 120, 150, 20, 13);

    let mut mtt_subtitle = build_label("Hide window to system tray.", x + 25, y + 138, 200, 20, 11);
    mtt_subtitle.set_label_color(Color::from_rgb(150, 150, 150));

    let mut minimize_to_tray_btn = button::CheckButton::default()
        .with_size(0, 0)
        .with_pos(0, 0);
    minimize_to_tray_btn.hide();
    let _mtt_seg = create_custom_toggle(
        x + w - 110,
        y + 123,
        initial_minimize_to_tray,
        &mut minimize_to_tray_btn,
    );

    let _pwc_title = build_title("Pause on Window Change", x + 25, y + 165, 180, 20, 13);

    let mut pwc_subtitle = build_label(
        "Pause clicking when switching windows.",
        x + 25,
        y + 183,
        220,
        20,
        11,
    );
    pwc_subtitle.set_label_color(Color::from_rgb(150, 150, 150));

    let mut pause_on_window_change_btn = button::CheckButton::default()
        .with_size(0, 0)
        .with_pos(0, 0);
    pause_on_window_change_btn.hide();
    let _pwc_seg = create_custom_toggle(
        x + w - 110,
        y + 168,
        initial_pause_on_window_change,
        &mut pause_on_window_change_btn,
    );

    card.end();

    let mut hk_card = group::Group::default()
        .with_size(w - 20, 68)
        .with_pos(x + 10, y + 250);
    hk_card.set_frame(FrameType::RFlatBox);
    hk_card.set_color(Color::from_rgb(20, 20, 20));

    let _hk_title = build_title("Start/Stop Hotkey", x + 25, y + 258, 150, 20, 13);

    let mut hk_subtitle = build_label(
        "Click to assign a new hotkey.",
        x + 25,
        y + 276,
        200,
        20,
        11,
    );
    hk_subtitle.set_label_color(Color::from_rgb(150, 150, 150));

    let mut hotkey_btn = button::Button::default()
        .with_size(80, 24)
        .with_pos(x + w - 110, y + 263);
    hotkey_btn.set_color(col(CLR_WIDGET));
    hotkey_btn.set_selection_color(col(CLR_WIDGET));
    hotkey_btn.set_label_color(Color::White);
    hotkey_btn.set_label_font(Font::HelveticaBold);
    hotkey_btn.set_label_size(12);
    hotkey_btn.set_frame(FrameType::RFlatBox);
    hotkey_btn.clear_visible_focus();

    let current_hotkey = Arc::new(AtomicI32::new(initial_hotkey));
    hotkey_btn.set_label(&key_to_str(Key::from_i32(initial_hotkey)));

    let is_listening = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let skip_next_hotkey = Arc::new(std::sync::atomic::AtomicBool::new(false));

    let is_listening_cb = is_listening.clone();
    let mut btn_cb = hotkey_btn.clone();
    hotkey_btn.set_callback(move |_| {
        if !is_listening_cb.load(Ordering::SeqCst) {
            is_listening_cb.store(true, Ordering::SeqCst);
            btn_cb.set_label("Press Key");
            btn_cb.set_color(col(CLR_GREEN));
            btn_cb.set_selection_color(col(CLR_GREEN));
            btn_cb.redraw();
            let _ = btn_cb.take_focus();
        }
    });

    let is_listening_h = is_listening.clone();
    let current_hotkey_h = current_hotkey.clone();
    let skip_next_h = skip_next_hotkey.clone();
    let tx_hk = tx.clone();
    hotkey_btn.handle(move |btn, ev| {
        let listening = is_listening_h.load(Ordering::SeqCst);
        if listening {
            match ev {
                Event::Focus => true,
                Event::KeyDown | Event::Shortcut => {
                    let key = app::event_key();
                    current_hotkey_h.store(key.bits(), Ordering::SeqCst);
                    btn.set_label(&key_to_str(key));
                    btn.set_color(col(CLR_WIDGET));
                    btn.set_selection_color(col(CLR_WIDGET));
                    is_listening_h.store(false, Ordering::SeqCst);

                    skip_next_h.store(true, Ordering::SeqCst);
                    btn.redraw();

                    if let Some(mut win) = app::first_window() {
                        let _ = win.take_focus();
                    }
                    let _ = tx_hk.send(());
                    true
                }
                Event::Unfocus => {
                    let key = Key::from_i32(current_hotkey_h.load(Ordering::SeqCst));
                    btn.set_label(&key_to_str(key));
                    btn.set_color(col(CLR_WIDGET));
                    btn.set_selection_color(col(CLR_WIDGET));
                    is_listening_h.store(false, Ordering::SeqCst);
                    btn.redraw();
                    true
                }
                _ => false,
            }
        } else {
            match ev {
                Event::Enter => {
                    btn.set_color(Color::from_rgb(45, 45, 45));
                    btn.set_selection_color(Color::from_rgb(45, 45, 45));
                    btn.redraw();
                    true
                }
                Event::Leave => {
                    btn.set_color(col(CLR_WIDGET));
                    btn.set_selection_color(col(CLR_WIDGET));
                    btn.redraw();
                    true
                }
                _ => false,
            }
        }
    });

    hk_card.end();

    settings_group.end();
    settings_group.hide();

    SettingsHandles {
        group: settings_group,
        always_on_top_btn,
        minimize_to_tray_btn,
        pause_on_window_change_btn,
        current_hotkey,
        is_listening,
        skip_next_hotkey,
    }
}

fn key_to_str(key: Key) -> String {
    match key {
        Key::F1 => "F1".to_string(),
        Key::F2 => "F2".to_string(),
        Key::F3 => "F3".to_string(),
        Key::F4 => "F4".to_string(),
        Key::F5 => "F5".to_string(),
        Key::F6 => "F6".to_string(),
        Key::F7 => "F7".to_string(),
        Key::F8 => "F8".to_string(),
        Key::F9 => "F9".to_string(),
        Key::F10 => "F10".to_string(),
        Key::F11 => "F11".to_string(),
        Key::F12 => "F12".to_string(),
        Key::Escape => "Escape".to_string(),
        Key::Tab => "Tab".to_string(),
        Key::Enter => "Enter".to_string(),
        Key::BackSpace => "Backspace".to_string(),
        Key::Insert => "Insert".to_string(),
        Key::Delete => "Delete".to_string(),
        Key::Home => "Home".to_string(),
        Key::End => "End".to_string(),
        Key::PageUp => "PgUp".to_string(),
        Key::PageDown => "PgDn".to_string(),
        Key::Up => "Up".to_string(),
        Key::Down => "Down".to_string(),
        Key::Left => "Left".to_string(),
        Key::Right => "Right".to_string(),
        Key::ControlL => "LCtrl".to_string(),
        Key::ControlR => "RCtrl".to_string(),
        Key::ShiftL => "LShift".to_string(),
        Key::ShiftR => "RShift".to_string(),
        Key::AltL => "LAlt".to_string(),
        Key::AltR => "RAlt".to_string(),
        Key::MetaL => "LWin".to_string(),
        Key::MetaR => "RWin".to_string(),
        Key::CapsLock => "CapsLock".to_string(),
        _ => {
            let bits = key.bits();
            if bits == 32 {
                "Space".to_string()
            } else if let Some(c) = char::from_u32(bits as u32) {
                if c.is_ascii_alphanumeric() || c.is_ascii_punctuation() {
                    c.to_string().to_uppercase()
                } else {
                    format!("Key {}", bits)
                }
            } else {
                format!("Key {}", bits)
            }
        }
    }
}
