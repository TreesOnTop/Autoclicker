#![windows_subsystem = "windows"]

mod platform;
mod ui;

use fltk::{app, prelude::*};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use ui::update_button_style;

fn main() {
    let app = app::App::default().with_scheme(app::Scheme::Gtk);

    // Set modern global border radius for widgets (inputs, dropdowns, buttons)
    app::set_frame_border_radius_max(6);

    // Disable visible focus globally to prevent focus borders on focused elements
    app::set_visible_focus(false);

    // Build all UI widgets and retrieve handles needed for callbacks
    let ui::UiHandles {
        mut wind,
        mut close_btn,
        mut min_btn,
        mut always_on_top_btn,
        mut start_stop_btn,
        status_badge,
    } = ui::build_ui();

    // --- DRAG-TO-MOVE ---
    ui::attach_drag_handler(&mut wind);

    // --- CLOSE BUTTON ---
    let mut wind_close = wind.clone();
    close_btn.set_callback(move |_| {
        wind_close.hide();
        app::quit();
    });

    close_btn.handle(move |btn, ev| match ev {
        fltk::enums::Event::Enter => {
            btn.set_color(ui::col(ui::CLR_RED));
            btn.redraw();
            true
        }
        fltk::enums::Event::Leave => {
            btn.set_color(ui::col(ui::CLR_TITLEBAR));
            btn.redraw();
            true
        }
        _ => false,
    });

    // --- MINIMIZE BUTTON ---
    let mut wind_min = wind.clone();
    min_btn.set_callback(move |_| {
        wind_min.iconize();
    });

    min_btn.handle(move |btn, ev| match ev {
        fltk::enums::Event::Enter => {
            btn.set_color(ui::col(ui::CLR_TITLEBAR_HOVER));
            btn.redraw();
            true
        }
        fltk::enums::Event::Leave => {
            btn.set_color(ui::col(ui::CLR_TITLEBAR));
            btn.redraw();
            true
        }
        _ => false,
    });

    // --- ALWAYS ON TOP ---
    let mut wind_top = wind.clone();
    always_on_top_btn.set_callback(move |btn| {
        platform::set_window_topmost(&mut wind_top, btn.value());
    });

    // --- START / STOP TOGGLE ---
    let is_active = Arc::new(AtomicBool::new(false));

    let is_active_toggle = is_active.clone();
    let mut btn_clone = start_stop_btn.clone();
    let mut badge_clone = status_badge.clone();
    start_stop_btn.set_callback(move |_| {
        let new_state = !is_active_toggle.load(Ordering::SeqCst);
        is_active_toggle.store(new_state, Ordering::SeqCst);
        update_button_style(&mut btn_clone, &mut badge_clone, new_state);
    });

    // --- START / STOP HOVER ---
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

    wind.show();

    // Apply native Windows 11 rounded corners + accent border, and taskbar visibility
    platform::apply_windows_style(&mut wind);
    platform::show_in_taskbar(&mut wind);

    app.run().unwrap();
}
