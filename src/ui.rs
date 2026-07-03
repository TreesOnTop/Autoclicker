use fltk::{app, button, enums::*, frame, prelude::*, window};

use crate::pages::clicker::build_clicker_page;
use crate::pages::processes::build_processes_page;
use crate::pages::settings::build_settings_page;

pub const CLR_BACKGROUND: (u8, u8, u8) = (11, 11, 11);
pub const CLR_TITLEBAR: (u8, u8, u8) = (26, 26, 26);
pub const CLR_BORDER: (u8, u8, u8) = (51, 51, 51);
pub const CLR_WIDGET: (u8, u8, u8) = (30, 30, 30);
pub const CLR_BADGE_INACTIVE: (u8, u8, u8) = (45, 45, 45);
pub const CLR_LABEL: (u8, u8, u8) = (180, 180, 180);
pub const CLR_FOOTER: (u8, u8, u8) = (100, 100, 100);
pub const CLR_GREEN: (u8, u8, u8) = (58, 150, 109);
pub const CLR_GREEN_HOVER: (u8, u8, u8) = (70, 180, 130);
pub const CLR_RED: (u8, u8, u8) = (209, 73, 73);
pub const CLR_RED_HOVER: (u8, u8, u8) = (230, 90, 90);
pub const CLR_TITLEBAR_HOVER: (u8, u8, u8) = (90, 90, 90);
pub const CLR_TAB_ACTIVE: (u8, u8, u8) = (70, 70, 70);

#[inline(always)]
pub fn col(c: (u8, u8, u8)) -> Color {
    Color::from_rgb(c.0, c.1, c.2)
}

pub struct UiHandles {
    pub wind: window::Window,
    pub close_btn: button::Button,
    pub min_btn: button::Button,
    pub always_on_top_btn: button::CheckButton,
    pub minimize_to_tray_btn: button::CheckButton,
    pub pause_on_window_change_btn: button::CheckButton,
    pub start_stop_btn: button::Button,
    pub status_badge: frame::Frame,
    pub current_hotkey: std::sync::Arc<std::sync::atomic::AtomicI32>,
    pub is_listening: std::sync::Arc<std::sync::atomic::AtomicBool>,

    pub skip_next_hotkey: std::sync::Arc<std::sync::atomic::AtomicBool>,

    pub interval_ms: std::sync::Arc<std::sync::atomic::AtomicI32>,

    pub click_type_index: std::sync::Arc<std::sync::atomic::AtomicI32>,
    pub filter_mode_choice: fltk::menu::Choice,
    pub processes_browser: fltk::browser::HoldBrowser,
}

pub fn build_ui(
    settings: &crate::settings_io::AppSettings,
    tx: std::sync::mpsc::Sender<()>,
) -> UiHandles {
    let (sw, sh) = app::screen_size();
    let win_w = 390;
    let win_h = 365;
    let x = ((sw as i32) - win_w) / 2;
    let y = ((sh as i32) - win_h) / 2;
    let mut wind = window::Window::default()
        .with_size(win_w, win_h)
        .with_pos(x, y)
        .with_label("Tree AutoClicker");
    wind.set_border(false);
    wind.set_color(col(CLR_BACKGROUND));

    let mut title_bar_bg = frame::Frame::default().with_size(win_w, 35).with_pos(0, 0);
    title_bar_bg.set_color(col(CLR_TITLEBAR));
    title_bar_bg.set_frame(FrameType::FlatBox);

    let mut title_text = frame::Frame::default().with_size(160, 35).with_pos(115, 0);
    title_text.set_label("Tree AutoClicker");
    title_text.set_label_color(Color::White);
    title_text.set_label_font(Font::HelveticaBold);
    title_text.set_label_size(13);
    title_text.set_align(Align::Center | Align::Inside);

    let tab_sz = 25;
    let tab_y = (35 - tab_sz) / 2;

    let mut tab_settings = button::Button::default()
        .with_size(tab_sz, tab_sz)
        .with_pos(5, tab_y);
    tab_settings.set_label("⚙️");
    tab_settings.set_label_color(Color::White);
    tab_settings.set_label_size(14);
    tab_settings.set_color(col(CLR_TITLEBAR));
    tab_settings.set_selection_color(col(CLR_TITLEBAR));
    tab_settings.set_frame(FrameType::RFlatBox);
    tab_settings.clear_visible_focus();

    let mut tab_clicker = button::Button::default()
        .with_size(tab_sz, tab_sz)
        .with_pos(35, tab_y);
    tab_clicker.set_label("🖱️");
    tab_clicker.set_label_color(Color::White);
    tab_clicker.set_label_size(14);
    tab_clicker.set_color(col(CLR_TAB_ACTIVE));
    tab_clicker.set_selection_color(col(CLR_TAB_ACTIVE));
    tab_clicker.set_frame(FrameType::RFlatBox);
    tab_clicker.clear_visible_focus();

    let mut tab_processes = button::Button::default()
        .with_size(tab_sz, tab_sz)
        .with_pos(65, tab_y);
    tab_processes.set_label("📋");
    tab_processes.set_label_color(Color::White);
    tab_processes.set_label_size(14);
    tab_processes.set_color(col(CLR_TITLEBAR));
    tab_processes.set_selection_color(col(CLR_TITLEBAR));
    tab_processes.set_frame(FrameType::RFlatBox);
    tab_processes.clear_visible_focus();

    let mut min_btn = button::Button::default().with_size(30, 25).with_pos(320, 5);
    min_btn.set_label("—");
    min_btn.set_label_color(Color::White);
    min_btn.set_label_size(12);
    min_btn.set_color(col(CLR_TITLEBAR));
    min_btn.set_selection_color(Color::from_rgb(80, 80, 80));
    min_btn.set_frame(FrameType::RFlatBox);
    min_btn.clear_visible_focus();

    let mut close_btn = button::Button::default().with_size(30, 25).with_pos(355, 5);
    close_btn.set_label("×");
    close_btn.set_label_color(Color::White);
    close_btn.set_label_size(18);
    close_btn.set_color(col(CLR_TITLEBAR));
    close_btn.set_selection_color(col(CLR_RED_HOVER));
    close_btn.set_frame(FrameType::RFlatBox);
    close_btn.clear_visible_focus();

    let mut border = frame::Frame::default().with_size(win_w, 2).with_pos(0, 35);
    border.set_color(col(CLR_BORDER));
    border.set_frame(FrameType::FlatBox);

    let mut content_area = frame::Frame::default()
        .with_size(win_w, 328)
        .with_pos(0, 37);
    content_area.set_color(col(CLR_BACKGROUND));
    content_area.set_frame(FrameType::FlatBox);

    let clicker_handles = build_clicker_page(
        0,
        37,
        win_w,
        328,
        settings.interval_ms,
        settings.click_type_index,
        tx.clone(),
    );
    let settings_handles = build_settings_page(
        0,
        37,
        win_w,
        328,
        settings.always_on_top,
        settings.minimize_to_tray,
        settings.pause_on_window_change,
        settings.current_hotkey,
        tx.clone(),
    );
    let processes_handles = build_processes_page(
        0,
        37,
        win_w,
        328,
        settings.filter_mode,
        settings.processes.clone(),
        tx.clone(),
    );

    let mut footer = frame::Frame::default()
        .with_size(win_w, 25)
        .with_pos(0, 337);
    footer.set_label("Tree AutoClicker");
    footer.set_label_color(col(CLR_FOOTER));
    footer.set_label_font(Font::HelveticaItalic);
    footer.set_label_size(11);
    footer.set_align(Align::Center | Align::Inside);

    wind.end();

    let tabs = vec![
        tab_settings.clone(),
        tab_clicker.clone(),
        tab_processes.clone(),
    ];
    let groups = vec![
        settings_handles.group.clone(),
        clicker_handles.group.clone(),
        processes_handles.group.clone(),
    ];

    for i in 0..tabs.len() {
        let mut tab = tabs[i].clone();
        let tabs_clone = tabs.clone();
        let mut groups_clone = groups.clone();
        let mut wind_clone = wind.clone();

        tab.set_callback(move |_| {
            for (j, g) in groups_clone.iter_mut().enumerate() {
                if i == j {
                    g.show();
                } else {
                    g.hide();
                }
            }
            for (j, mut t) in tabs_clone.clone().into_iter().enumerate() {
                let color = if i == j {
                    col(CLR_TAB_ACTIVE)
                } else {
                    col(CLR_TITLEBAR)
                };
                t.set_color(color);
                t.set_selection_color(color);
            }
            wind_clone.redraw();
        });
    }

    let _ = (
        title_bar_bg,
        title_text,
        border,
        content_area,
        tab_clicker,
        tab_settings,
        tab_processes,
        footer,
    );

    UiHandles {
        wind,
        close_btn,
        min_btn,
        always_on_top_btn: settings_handles.always_on_top_btn,
        minimize_to_tray_btn: settings_handles.minimize_to_tray_btn,
        pause_on_window_change_btn: settings_handles.pause_on_window_change_btn,
        start_stop_btn: clicker_handles.start_stop_btn,
        status_badge: clicker_handles.status_badge,
        current_hotkey: settings_handles.current_hotkey,
        is_listening: settings_handles.is_listening,
        skip_next_hotkey: settings_handles.skip_next_hotkey,
        interval_ms: clicker_handles.interval_ms,
        click_type_index: clicker_handles.click_type_index,
        filter_mode_choice: processes_handles.filter_mode_choice,
        processes_browser: processes_handles.processes_browser,
    }
}

pub fn update_button_style(btn: &mut button::Button, badge: &mut frame::Frame, active: bool) {
    if active {
        btn.set_color(col(CLR_RED));
        btn.set_selection_color(col(CLR_RED_HOVER));
        btn.set_label("STOP");
        badge.set_color(col(CLR_RED));
        badge.set_label("ACTIVE");
    } else {
        btn.set_color(col(CLR_GREEN));
        btn.set_selection_color(col(CLR_GREEN_HOVER));
        btn.set_label("START");
        badge.set_color(col(CLR_BADGE_INACTIVE));
        badge.set_label("INACTIVE");
    }
    btn.redraw();
    badge.redraw();
}

pub fn setup_window_events(
    wind: &mut window::Window,
    _is_listening: std::sync::Arc<std::sync::atomic::AtomicBool>,
) {
    let mut drag_offset_x = 0;
    let mut drag_offset_y = 0;
    let mut is_dragging = false;

    wind.handle(move |w, ev| match ev {
        Event::Push => {
            let ex = app::event_x();
            let ey = app::event_y();
            if ey >= 0 && ey <= 35 && ex >= 0 && ex < 320 {
                drag_offset_x = app::event_x_root() - w.x();
                drag_offset_y = app::event_y_root() - w.y();
                is_dragging = true;
                true
            } else {
                is_dragging = false;
                false
            }
        }
        Event::Drag => {
            if is_dragging {
                w.set_pos(
                    app::event_x_root() - drag_offset_x,
                    app::event_y_root() - drag_offset_y,
                );
                true
            } else {
                false
            }
        }
        Event::Released => {
            is_dragging = false;
            true
        }
        _ => false,
    });
}

pub fn add_hover_effect<T: WidgetExt + WidgetBase + 'static>(
    mut widget: T,
    normal_color: Color,
    hover_color: Color,
) {
    widget.handle(move |w: &mut T, ev| match ev {
        Event::Enter => {
            w.set_color(hover_color);
            match w.window() {
                Some(mut win) => {
                    win.redraw();
                }
                _ => {
                    w.redraw();
                }
            }
            true
        }
        Event::Leave => {
            w.set_color(normal_color);
            match w.window() {
                Some(mut win) => {
                    win.redraw();
                }
                _ => {
                    w.redraw();
                }
            }
            true
        }
        _ => false,
    });
}
