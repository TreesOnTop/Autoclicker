#![windows_subsystem = "windows"]

mod clicker_engine;
mod pages;
mod platform;
mod settings_io;
mod ui;

use fltk::{app, prelude::*};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use ui::update_button_style;

#[derive(Clone)]
struct SettingsSaver {
    always_on_top_btn: fltk::button::CheckButton,
    minimize_to_tray_btn: fltk::button::CheckButton,
    pause_on_window_change_btn: fltk::button::CheckButton,
    current_hotkey: Arc<std::sync::atomic::AtomicI32>,
    interval_ms: Arc<std::sync::atomic::AtomicI32>,
    click_type_index: Arc<std::sync::atomic::AtomicI32>,
    filter_mode_choice: fltk::menu::Choice,
    processes_browser: fltk::browser::HoldBrowser,
}

impl SettingsSaver {
    fn save(&self) {
        let always_on_top = self.always_on_top_btn.value();
        let minimize_to_tray = self.minimize_to_tray_btn.value();
        let pause_on_window_change = self.pause_on_window_change_btn.value();
        let current_hotkey = self.current_hotkey.load(Ordering::SeqCst);
        let interval_ms = self.interval_ms.load(Ordering::SeqCst);
        let click_type_index = self.click_type_index.load(Ordering::SeqCst);
        let filter_mode = self.filter_mode_choice.value();

        let mut processes = Vec::new();
        let size = self.processes_browser.size();
        for i in 1..=size {
            if let Some(text) = self.processes_browser.text(i) {
                processes.push(text);
            }
        }

        let settings = settings_io::AppSettings {
            always_on_top,
            minimize_to_tray,
            pause_on_window_change,
            current_hotkey,
            interval_ms,
            click_type_index,
            filter_mode,
            processes,
        };
        settings_io::save_settings(&settings);
    }
}

fn main() {
    let settings = settings_io::load_settings();
    let (tx, rx) = std::sync::mpsc::channel::<()>();

    let app = app::App::default().with_scheme(app::Scheme::Gtk);

    app::set_frame_border_radius_max(6);

    app::set_visible_focus(false);

    let ui::UiHandles {
        mut wind,
        mut close_btn,
        mut min_btn,
        mut always_on_top_btn,
        mut minimize_to_tray_btn,
        mut pause_on_window_change_btn,
        mut start_stop_btn,
        status_badge,
        current_hotkey,
        is_listening,
        skip_next_hotkey,
        interval_ms,
        click_type_index,
        filter_mode_choice,
        processes_browser,
    } = ui::build_ui(&settings, tx.clone());

    let mut wind_close = wind.clone();
    close_btn.set_callback(move |_| {
        wind_close.hide();
        app::quit();
    });

    ui::add_hover_effect(
        close_btn.clone(),
        ui::col(ui::CLR_TITLEBAR),
        ui::col(ui::CLR_RED),
    );

    let mut wind_min = wind.clone();
    let min_to_tray = minimize_to_tray_btn.clone();
    min_btn.set_callback(move |btn| {
        btn.set_color(ui::col(ui::CLR_TITLEBAR));
        btn.window().unwrap().redraw();
        if min_to_tray.value() {
            platform::hide_window(&mut wind_min);
        } else {
            platform::minimize_window(&mut wind_min);
        }
    });

    ui::add_hover_effect(
        min_btn.clone(),
        ui::col(ui::CLR_TITLEBAR),
        ui::col(ui::CLR_TITLEBAR_HOVER),
    );

    let mut wind_top = wind.clone();
    let tx_top = tx.clone();
    always_on_top_btn.set_callback(move |btn| {
        platform::set_window_topmost(&mut wind_top, btn.value());
        let _ = tx_top.send(());
    });

    let tx_mtt = tx.clone();
    minimize_to_tray_btn.set_callback(move |_| {
        let _ = tx_mtt.send(());
    });

    let tx_pwc = tx.clone();
    pause_on_window_change_btn.set_callback(move |_| {
        let _ = tx_pwc.send(());
    });

    let is_active = Arc::new(AtomicBool::new(false));

    let is_active_toggle = is_active.clone();
    let interval_ms_toggle = interval_ms.clone();
    let click_type_index_toggle = click_type_index.clone();
    let mut btn_clone = start_stop_btn.clone();
    let mut badge_clone = status_badge.clone();
    start_stop_btn.set_callback(move |_| {
        let new_state = !is_active_toggle.load(Ordering::SeqCst);
        is_active_toggle.store(new_state, Ordering::SeqCst);
        if new_state {
            clicker_engine::start_from_atomics(
                interval_ms_toggle.clone(),
                click_type_index_toggle.clone(),
                is_active_toggle.clone(),
            );
        }
        update_button_style(&mut btn_clone, &mut badge_clone, new_state);
    });

    let is_active_hover = is_active.clone();
    let mut btn_hover = start_stop_btn.clone();
    start_stop_btn.handle(move |_, ev| {
        let active = is_active_hover.load(Ordering::SeqCst);
        match ev {
            fltk::enums::Event::Enter => {
                btn_hover.set_color(if active {
                    ui::col(ui::CLR_RED_HOVER)
                } else {
                    ui::col(ui::CLR_GREEN_HOVER)
                });
                btn_hover.redraw();
                true
            }
            fltk::enums::Event::Leave => {
                btn_hover.set_color(if active {
                    ui::col(ui::CLR_RED)
                } else {
                    ui::col(ui::CLR_GREEN)
                });
                btn_hover.redraw();
                true
            }
            _ => false,
        }
    });

    ui::setup_window_events(&mut wind, is_listening.clone());

    wind.show();

    platform::apply_windows_style(&mut wind);
    platform::show_in_taskbar(&mut wind);
    platform::set_window_topmost(&mut wind, always_on_top_btn.value());

    let (_hook_guard, key_rx) = platform::install_global_hook();

    let mut icon_rgba = Vec::with_capacity(16 * 16 * 4);
    for _ in 0..(16 * 16) {
        icon_rgba.extend_from_slice(&[255, 0, 0, 255]);
    }
    let tray_icon_img = tray_icon::Icon::from_rgba(icon_rgba, 16, 16).unwrap();
    let _tray_icon = tray_icon::TrayIconBuilder::new()
        .with_tooltip("Tree AutoClicker")
        .with_icon(tray_icon_img)
        .build()
        .unwrap();

    let refresh_rate = platform::get_monitor_refresh_rate();
    let frame_time = 1.0 / (refresh_rate as f64);

    let key_rx_mutex = Arc::new(std::sync::Mutex::new(key_rx));
    let rx_mutex = Arc::new(std::sync::Mutex::new(rx));
    let saver = SettingsSaver {
        always_on_top_btn: always_on_top_btn.clone(),
        minimize_to_tray_btn: minimize_to_tray_btn.clone(),
        pause_on_window_change_btn: pause_on_window_change_btn.clone(),
        current_hotkey: current_hotkey.clone(),
        interval_ms: interval_ms.clone(),
        click_type_index: click_type_index.clone(),
        filter_mode_choice,
        processes_browser,
    };
    schedule_poll(
        frame_time,
        key_rx_mutex,
        is_listening.clone(),
        current_hotkey.clone(),
        skip_next_hotkey.clone(),
        is_active.clone(),
        interval_ms.clone(),
        click_type_index.clone(),
        start_stop_btn.clone(),
        status_badge.clone(),
        wind.clone(),
        rx_mutex,
        saver,
        None,
    );

    app.run().unwrap();
}

fn schedule_poll(
    frame_time: f64,
    key_rx: Arc<std::sync::Mutex<std::sync::mpsc::Receiver<u32>>>,
    is_listening: Arc<AtomicBool>,
    current_hotkey: Arc<std::sync::atomic::AtomicI32>,
    skip_next_hk: Arc<AtomicBool>,
    is_active_hk: Arc<AtomicBool>,
    interval_ms_hk: Arc<std::sync::atomic::AtomicI32>,
    click_type_index_hk: Arc<std::sync::atomic::AtomicI32>,
    mut btn_hk: fltk::button::Button,
    mut badge_hk: fltk::frame::Frame,
    mut wind_tray: fltk::window::Window,
    settings_rx: Arc<std::sync::Mutex<std::sync::mpsc::Receiver<()>>>,
    saver: SettingsSaver,
    monitored_window: Option<*mut std::ffi::c_void>,
) {
    app::add_timeout3(frame_time, move |_| {
        let mut next_monitored_window = monitored_window;
        let is_active = is_active_hk.load(Ordering::SeqCst);
        if is_active {
            if saver.pause_on_window_change_btn.value() {
                let current_fg = platform::get_foreground_window();
                let my_hwnd = wind_tray.raw_handle();
                if current_fg != my_hwnd {
                    if let Some(target) = next_monitored_window {
                        if current_fg != target {
                            is_active_hk.store(false, Ordering::SeqCst);
                            update_button_style(&mut btn_hk, &mut badge_hk, false);
                            next_monitored_window = None;
                        }
                    } else {
                        next_monitored_window = Some(current_fg);
                    }
                }
            }
        } else {
            next_monitored_window = None;
        }

        if !is_listening.load(Ordering::SeqCst) {
            if let Ok(rx) = key_rx.try_lock() {
                while let Ok(vk) = rx.try_recv() {
                    let fltk_bits = vk_to_fltk_bits(vk);
                    if fltk_bits == current_hotkey.load(Ordering::SeqCst) {
                        if skip_next_hk.swap(false, Ordering::SeqCst) {
                            continue;
                        }
                        let new_state = !is_active_hk.load(Ordering::SeqCst);
                        is_active_hk.store(new_state, Ordering::SeqCst);
                        if new_state {
                            clicker_engine::start_from_atomics(
                                interval_ms_hk.clone(),
                                click_type_index_hk.clone(),
                                is_active_hk.clone(),
                            );
                        }
                        update_button_style(&mut btn_hk, &mut badge_hk, new_state);
                    }
                }
            }
        }

        while let Ok(event) = tray_icon::TrayIconEvent::receiver().try_recv() {
            if let tray_icon::TrayIconEvent::Click {
                button: tray_icon::MouseButton::Left,
                button_state: tray_icon::MouseButtonState::Up,
                ..
            } = event
            {
                platform::show_window(&mut wind_tray);
            }
        }

        if let Ok(rx) = settings_rx.try_lock() {
            let mut changed = false;
            while rx.try_recv().is_ok() {
                changed = true;
            }
            if changed {
                saver.save();
            }
        }

        schedule_poll(
            frame_time,
            key_rx.clone(),
            is_listening.clone(),
            current_hotkey.clone(),
            skip_next_hk.clone(),
            is_active_hk.clone(),
            interval_ms_hk.clone(),
            click_type_index_hk.clone(),
            btn_hk.clone(),
            badge_hk.clone(),
            wind_tray.clone(),
            settings_rx.clone(),
            saver.clone(),
            next_monitored_window,
        );
    });
}

fn vk_to_fltk_bits(vk: u32) -> i32 {
    match vk {
        0x70 => 0xFFBE,
        0x71 => 0xFFBF,
        0x72 => 0xFFC0,
        0x73 => 0xFFC1,
        0x74 => 0xFFC2,
        0x75 => 0xFFC3,
        0x76 => 0xFFC4,
        0x77 => 0xFFC5,
        0x78 => 0xFFC6,
        0x79 => 0xFFC7,
        0x7A => 0xFFC8,
        0x7B => 0xFFC9,

        0x1B => 0xFF1B,
        0x09 => 0xFF09,
        0x0D => 0xFF0D,
        0x08 => 0xFF08,
        0x2D => 0xFF63,
        0x2E => 0xFFFF,
        0x24 => 0xFF50,
        0x23 => 0xFF57,
        0x21 => 0xFF55,
        0x22 => 0xFF56,
        0x26 => 0xFF52,
        0x28 => 0xFF54,
        0x25 => 0xFF51,
        0x27 => 0xFF53,

        0xA2 => 0xFFE3,
        0xA3 => 0xFFE4,
        0xA0 => 0xFFE1,
        0xA1 => 0xFFE2,
        0xA4 => 0xFFE9,
        0xA5 => 0xFFEA,
        0x5B => 0xFFEB,
        0x5C => 0xFFEC,
        0x14 => 0xFFE5,

        0x20 => 32,

        v @ 0x41..=0x5A => (v + 0x20) as i32,

        v @ 0x30..=0x39 => v as i32,

        v => v as i32,
    }
}
