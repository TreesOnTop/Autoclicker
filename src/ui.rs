use fltk::{app, button, frame, window, input, menu, group, browser, prelude::*, enums::*};

// ---------------------------------------------------------------------------
// Color palette — change values here to restyle the whole UI at once.
// ---------------------------------------------------------------------------

/// Main window / content-area background.
pub const CLR_BACKGROUND:     (u8, u8, u8) = (11,  11,  11);
/// Title bar, minimize button, and close button background.
pub const CLR_TITLEBAR:       (u8, u8, u8) = (26,  26,  26);
/// Border between title bar and content area.
pub const CLR_BORDER:         (u8, u8, u8) = (51,  51,  51);
/// Input and dropdown background.
pub const CLR_WIDGET:         (u8, u8, u8) = (30,  30,  30);
/// Inactive status-badge background.
pub const CLR_BADGE_INACTIVE: (u8, u8, u8) = (45,  45,  45);
/// Secondary label text (field labels, checkbox).
pub const CLR_LABEL:          (u8, u8, u8) = (180, 180, 180);
/// Title-bar icon text (— ×).
pub const CLR_ICON:           (u8, u8, u8) = (200, 200, 200);
/// Footer / hint text.
pub const CLR_FOOTER:         (u8, u8, u8) = (100, 100, 100);
/// Start button and always-on-top checkmark.
pub const CLR_GREEN:          (u8, u8, u8) = (58,  150, 109);
/// Hover tint for the start button.
pub const CLR_GREEN_HOVER:    (u8, u8, u8) = (70,  180, 130);
/// Stop button, active badge, and close-button hover.
pub const CLR_RED:            (u8, u8, u8) = (209, 73,  73);
/// Hover tint for the stop button.
pub const CLR_RED_HOVER:      (u8, u8, u8) = (230, 90,  90);
/// Minimize button hover tint.
pub const CLR_TITLEBAR_HOVER: (u8, u8, u8) = (50,  50,  50);

/// Helper so call sites stay readable: `col(CLR_BACKGROUND)`.
#[inline(always)]
pub fn col(c: (u8, u8, u8)) -> Color {
    Color::from_rgb(c.0, c.1, c.2)
}

// ---------------------------------------------------------------------------

/// Handles to the widgets that need to be wired up with callbacks in main.
pub struct UiHandles {
    pub wind: window::Window,
    pub close_btn: button::Button,
    pub min_btn: button::Button,
    pub always_on_top_btn: button::CheckButton,
    pub start_stop_btn: button::Button,
    pub status_badge: frame::Frame,
}

/// Constructs all widgets and returns handles needed for event wiring.
pub fn build_ui() -> UiHandles {
    // --- WINDOW ---
    let (sw, sh) = app::screen_size();
    let win_w = 340;
    let win_h = 380;
    let x = ((sw as i32) - win_w) / 2;
    let y = ((sh as i32) - win_h) / 2;
    let mut wind = window::Window::default()
        .with_size(win_w, win_h)
        .with_pos(x, y)
        .with_label("Tree AutoClicker");
    wind.set_border(false);
    wind.set_color(col(CLR_BACKGROUND));

    // --- TITLE BAR ---
    let mut title_bar = frame::Frame::default()
        .with_size(340, 35)
        .with_pos(0, 0);
    title_bar.set_color(col(CLR_TITLEBAR));
    title_bar.set_frame(FrameType::FlatBox);
    title_bar.set_label("Tree AutoClicker");
    title_bar.set_label_color(Color::White);
    title_bar.set_label_font(Font::HelveticaBold);
    title_bar.set_label_size(13);
    title_bar.set_align(Align::Center | Align::Inside);

    // --- TAB BAR AREA ---
    let mut tab_bar = frame::Frame::default()
        .with_size(340, 35)
        .with_pos(0, 35);
    tab_bar.set_color(col(CLR_TITLEBAR));
    tab_bar.set_frame(FrameType::FlatBox);

    // --- TAB BUTTONS ---
    let mut tab_clicker = button::Button::default()
        .with_size(80, 26)
        .with_pos(20, 39);
    tab_clicker.set_label("Clicker");
    tab_clicker.set_label_font(Font::HelveticaBold);
    tab_clicker.set_label_size(12);
    tab_clicker.set_color(col(CLR_BACKGROUND));
    tab_clicker.set_label_color(Color::White);
    tab_clicker.set_frame(FrameType::RFlatBox);
    tab_clicker.clear_visible_focus();

    let mut tab_settings = button::Button::default()
        .with_size(80, 26)
        .with_pos(105, 39);
    tab_settings.set_label("Settings");
    tab_settings.set_label_font(Font::Helvetica);
    tab_settings.set_label_size(12);
    tab_settings.set_color(col(CLR_TITLEBAR));
    tab_settings.set_label_color(col(CLR_LABEL));
    tab_settings.set_frame(FrameType::RFlatBox);
    tab_settings.clear_visible_focus();

    let mut tab_processes = button::Button::default()
        .with_size(90, 26)
        .with_pos(190, 39);
    tab_processes.set_label("Processes");
    tab_processes.set_label_font(Font::Helvetica);
    tab_processes.set_label_size(12);
    tab_processes.set_color(col(CLR_TITLEBAR));
    tab_processes.set_label_color(col(CLR_LABEL));
    tab_processes.set_frame(FrameType::RFlatBox);
    tab_processes.clear_visible_focus();

    // --- MINIMIZE BUTTON ---
    let mut min_btn = button::Button::default()
        .with_size(35, 35)
        .with_pos(270, 0);
    min_btn.set_label("—");
    min_btn.set_label_color(col(CLR_ICON));
    min_btn.set_label_size(12);
    min_btn.set_color(col(CLR_TITLEBAR));
    min_btn.set_frame(FrameType::FlatBox);
    min_btn.clear_visible_focus();

    // --- CLOSE BUTTON ---
    let mut close_btn = button::Button::default()
        .with_size(35, 35)
        .with_pos(305, 0);
    close_btn.set_label("×");
    close_btn.set_label_color(col(CLR_ICON));
    close_btn.set_label_size(18);
    close_btn.set_color(col(CLR_TITLEBAR));
    close_btn.set_frame(FrameType::FlatBox);
    close_btn.clear_visible_focus();

    // --- BORDER ---
    let mut border = frame::Frame::default()
        .with_size(340, 2)
        .with_pos(0, 70);
    border.set_color(col(CLR_BORDER));
    border.set_frame(FrameType::FlatBox);

    // --- CONTENT BACKGROUND ---
    let mut content_area = frame::Frame::default()
        .with_size(340, 308)
        .with_pos(0, 72);
    content_area.set_color(col(CLR_BACKGROUND));
    content_area.set_frame(FrameType::FlatBox);

    // --- GROUP 1: CLICKER PAGE ---
    let clicker_group = group::Group::default()
        .with_size(340, 308)
        .with_pos(0, 72);

    let mut status_panel = frame::Frame::default()
        .with_size(300, 45)
        .with_pos(20, 85);
    status_panel.set_color(col(CLR_TITLEBAR));
    status_panel.set_frame(FrameType::RFlatBox);

    let mut status_title = frame::Frame::default()
        .with_size(80, 45)
        .with_pos(35, 85);
    status_title.set_label("Status:");
    status_title.set_label_color(col(CLR_LABEL));
    status_title.set_label_font(Font::HelveticaBold);
    status_title.set_label_size(13);
    status_title.set_align(Align::Left | Align::Inside);

    let mut status_badge = frame::Frame::default()
        .with_size(100, 26)
        .with_pos(205, 94);
    status_badge.set_frame(FrameType::RFlatBox);
    status_badge.set_color(col(CLR_BADGE_INACTIVE));
    status_badge.set_label("INACTIVE");
    status_badge.set_label_color(Color::White);
    status_badge.set_label_font(Font::HelveticaBold);
    status_badge.set_label_size(11);

    let mut interval_label = frame::Frame::default()
        .with_size(140, 30)
        .with_pos(20, 145);
    interval_label.set_label("Click Interval (ms)");
    interval_label.set_label_color(col(CLR_LABEL));
    interval_label.set_label_font(Font::Helvetica);
    interval_label.set_label_size(13);
    interval_label.set_align(Align::Left | Align::Inside);

    let mut interval_input = input::IntInput::default()
        .with_size(110, 30)
        .with_pos(210, 145);
    interval_input.set_value("100");
    interval_input.set_color(col(CLR_WIDGET));
    interval_input.set_text_color(Color::White);
    interval_input.set_text_font(Font::Helvetica);
    interval_input.set_text_size(13);
    interval_input.set_frame(FrameType::RFlatBox);
    interval_input.clear_visible_focus();

    let mut click_type_label = frame::Frame::default()
        .with_size(140, 30)
        .with_pos(20, 195);
    click_type_label.set_label("Click Type");
    click_type_label.set_label_color(col(CLR_LABEL));
    click_type_label.set_label_font(Font::Helvetica);
    click_type_label.set_label_size(13);
    click_type_label.set_align(Align::Left | Align::Inside);

    let mut click_type_choice = menu::Choice::default()
        .with_size(110, 30)
        .with_pos(210, 195);
    click_type_choice.set_color(col(CLR_WIDGET));
    click_type_choice.set_text_color(Color::White);
    click_type_choice.set_text_font(Font::Helvetica);
    click_type_choice.set_text_size(13);
    click_type_choice.set_frame(FrameType::RFlatBox);
    click_type_choice.add_choice("Left");
    click_type_choice.add_choice("Right");
    click_type_choice.add_choice("Middle");
    click_type_choice.set_value(0);
    click_type_choice.clear_visible_focus();

    let mut start_stop_btn = button::Button::default()
        .with_size(300, 50)
        .with_pos(20, 255);
    start_stop_btn.set_label("START");
    start_stop_btn.set_label_color(Color::White);
    start_stop_btn.set_label_font(Font::HelveticaBold);
    start_stop_btn.set_label_size(16);
    start_stop_btn.set_color(col(CLR_GREEN));
    start_stop_btn.set_frame(FrameType::RFlatBox);
    start_stop_btn.clear_visible_focus();

    clicker_group.end();

    // --- GROUP 2: SETTINGS PAGE ---
    let mut settings_group = group::Group::default()
        .with_size(340, 308)
        .with_pos(0, 72);

    let mut always_on_top_btn = button::CheckButton::default()
        .with_size(300, 30)
        .with_pos(20, 85);
    always_on_top_btn.set_label("Always on Top");
    always_on_top_btn.set_label_color(col(CLR_LABEL));
    always_on_top_btn.set_label_font(Font::Helvetica);
    always_on_top_btn.set_label_size(13);
    always_on_top_btn.set_selection_color(col(CLR_GREEN));
    always_on_top_btn.clear_visible_focus();

    let mut hotkey_label = frame::Frame::default()
        .with_size(140, 30)
        .with_pos(20, 135);
    hotkey_label.set_label("Start/Stop Hotkey");
    hotkey_label.set_label_color(col(CLR_LABEL));
    hotkey_label.set_label_font(Font::Helvetica);
    hotkey_label.set_label_size(13);
    hotkey_label.set_align(Align::Left | Align::Inside);

    let mut hotkey_choice = menu::Choice::default()
        .with_size(110, 30)
        .with_pos(210, 135);
    hotkey_choice.set_color(col(CLR_WIDGET));
    hotkey_choice.set_text_color(Color::White);
    hotkey_choice.set_text_font(Font::Helvetica);
    hotkey_choice.set_text_size(13);
    hotkey_choice.set_frame(FrameType::RFlatBox);
    hotkey_choice.add_choice("F1");
    hotkey_choice.add_choice("F2");
    hotkey_choice.add_choice("F3");
    hotkey_choice.add_choice("F4");
    hotkey_choice.add_choice("F5");
    hotkey_choice.add_choice("F6");
    hotkey_choice.add_choice("F7");
    hotkey_choice.add_choice("F8");
    hotkey_choice.add_choice("F9");
    hotkey_choice.add_choice("F10");
    hotkey_choice.add_choice("F11");
    hotkey_choice.add_choice("F12");
    hotkey_choice.set_value(9);
    hotkey_choice.clear_visible_focus();

    settings_group.end();
    settings_group.hide();

    // --- GROUP 3: PROCESSES PAGE ---
    let mut processes_group = group::Group::default()
        .with_size(340, 308)
        .with_pos(0, 72);

    let mut mode_label = frame::Frame::default()
        .with_size(100, 30)
        .with_pos(20, 85);
    mode_label.set_label("Filter Mode");
    mode_label.set_label_color(col(CLR_LABEL));
    mode_label.set_label_font(Font::Helvetica);
    mode_label.set_label_size(13);
    mode_label.set_align(Align::Left | Align::Inside);

    let mut mode_choice = menu::Choice::default()
        .with_size(180, 30)
        .with_pos(140, 85);
    mode_choice.set_color(col(CLR_WIDGET));
    mode_choice.set_text_color(Color::White);
    mode_choice.set_text_font(Font::Helvetica);
    mode_choice.set_text_size(13);
    mode_choice.set_frame(FrameType::RFlatBox);
    mode_choice.add_choice("Blacklist (Disable click)");
    mode_choice.add_choice("Whitelist (Enable click)");
    mode_choice.set_value(0);
    mode_choice.clear_visible_focus();

    let mut proc_input = input::Input::default()
        .with_size(210, 30)
        .with_pos(20, 130);
    proc_input.set_color(col(CLR_WIDGET));
    proc_input.set_text_color(Color::White);
    proc_input.set_text_font(Font::Helvetica);
    proc_input.set_text_size(13);
    proc_input.set_frame(FrameType::RFlatBox);
    proc_input.clear_visible_focus();
    
    let mut add_btn = button::Button::default()
        .with_size(80, 30)
        .with_pos(240, 130);
    add_btn.set_label("Add");
    add_btn.set_label_font(Font::HelveticaBold);
    add_btn.set_label_size(13);
    add_btn.set_color(col(CLR_GREEN));
    add_btn.set_label_color(Color::White);
    add_btn.set_frame(FrameType::RFlatBox);
    add_btn.clear_visible_focus();

    let mut list_browser = browser::HoldBrowser::default()
        .with_size(210, 140)
        .with_pos(20, 175);
    list_browser.set_color(col(CLR_WIDGET));
    list_browser.set_text_size(13);
    list_browser.set_frame(FrameType::RFlatBox);
    list_browser.clear_visible_focus();
    list_browser.add("notepad.exe");
    list_browser.add("chrome.exe");

    let mut remove_btn = button::Button::default()
        .with_size(80, 30)
        .with_pos(240, 175);
    remove_btn.set_label("Remove");
    remove_btn.set_label_font(Font::HelveticaBold);
    remove_btn.set_label_size(13);
    remove_btn.set_color(col(CLR_RED));
    remove_btn.set_label_color(Color::White);
    remove_btn.set_frame(FrameType::RFlatBox);
    remove_btn.clear_visible_focus();

    processes_group.end();
    processes_group.hide();

    // --- FOOTER ---
    let mut footer = frame::Frame::default()
        .with_size(300, 25)
        .with_pos(20, 337);
    footer.set_label("Tree AutoClicker");
    footer.set_label_color(col(CLR_FOOTER));
    footer.set_label_font(Font::HelveticaItalic);
    footer.set_label_size(11);
    footer.set_align(Align::Center | Align::Inside);

    wind.end();

    // --- TAB CALLBACKS ---
    let mut cg_c = clicker_group.clone();
    let mut sg_c = settings_group.clone();
    let mut pg_c = processes_group.clone();
    let mut tc_c = tab_clicker.clone();
    let mut ts_c = tab_settings.clone();
    let mut tp_c = tab_processes.clone();
    let mut w_c = wind.clone();
    tab_clicker.set_callback(move |_| {
        cg_c.show();
        sg_c.hide();
        pg_c.hide();
        
        tc_c.set_color(col(CLR_BACKGROUND));
        tc_c.set_label_color(Color::White);
        tc_c.set_label_font(Font::HelveticaBold);

        ts_c.set_color(col(CLR_TITLEBAR));
        ts_c.set_label_color(col(CLR_LABEL));
        ts_c.set_label_font(Font::Helvetica);

        tp_c.set_color(col(CLR_TITLEBAR));
        tp_c.set_label_color(col(CLR_LABEL));
        tp_c.set_label_font(Font::Helvetica);
        
        w_c.redraw();
    });

    let mut cg_s = clicker_group.clone();
    let mut sg_s = settings_group.clone();
    let mut pg_s = processes_group.clone();
    let mut tc_s = tab_clicker.clone();
    let mut ts_s = tab_settings.clone();
    let mut tp_s = tab_processes.clone();
    let mut w_s = wind.clone();
    tab_settings.set_callback(move |_| {
        cg_s.hide();
        sg_s.show();
        pg_s.hide();
        
        tc_s.set_color(col(CLR_TITLEBAR));
        tc_s.set_label_color(col(CLR_LABEL));
        tc_s.set_label_font(Font::Helvetica);

        ts_s.set_color(col(CLR_BACKGROUND));
        ts_s.set_label_color(Color::White);
        ts_s.set_label_font(Font::HelveticaBold);

        tp_s.set_color(col(CLR_TITLEBAR));
        tp_s.set_label_color(col(CLR_LABEL));
        tp_s.set_label_font(Font::Helvetica);
        
        w_s.redraw();
    });

    let mut cg_p = clicker_group.clone();
    let mut sg_p = settings_group.clone();
    let mut pg_p = processes_group.clone();
    let mut tc_p = tab_clicker.clone();
    let mut ts_p = tab_settings.clone();
    let mut tp_p = tab_processes.clone();
    let mut w_p = wind.clone();
    tab_processes.set_callback(move |_| {
        cg_p.hide();
        sg_p.hide();
        pg_p.show();
        
        tc_p.set_color(col(CLR_TITLEBAR));
        tc_p.set_label_color(col(CLR_LABEL));
        tc_p.set_label_font(Font::Helvetica);

        ts_p.set_color(col(CLR_TITLEBAR));
        ts_p.set_label_color(col(CLR_LABEL));
        ts_p.set_label_font(Font::Helvetica);

        tp_p.set_color(col(CLR_BACKGROUND));
        tp_p.set_label_color(Color::White);
        tp_p.set_label_font(Font::HelveticaBold);
        
        w_p.redraw();
    });

    // --- ADD/REMOVE PROCESS CALLBACKS ---
    let mut browser_clone = list_browser.clone();
    let input_clone = proc_input.clone();
    add_btn.set_callback(move |_| {
        let val = input_clone.value().trim().to_string();
        if !val.is_empty() {
            browser_clone.add(&val);
            let mut input_mut = input_clone.clone();
            input_mut.set_value("");
        }
    });

    let mut browser_clone2 = list_browser.clone();
    remove_btn.set_callback(move |_| {
        let selected = browser_clone2.value();
        if selected > 0 {
            browser_clone2.remove(selected);
        }
    });

    // Suppress unused-variable warnings for widgets not returned
    let _ = (title_bar, tab_bar, border, content_area, status_panel, status_title,
             interval_label, interval_input, click_type_label, click_type_choice,
             hotkey_label, hotkey_choice, mode_label, mode_choice, proc_input,
             add_btn, list_browser, remove_btn, tab_clicker, tab_settings, tab_processes,
             clicker_group, settings_group, processes_group, footer);

    UiHandles {
        wind,
        close_btn,
        min_btn,
        always_on_top_btn,
        start_stop_btn,

        status_badge,
    }
}

/// Updates the start/stop button and status badge to reflect the current active state.
pub fn update_button_style(btn: &mut button::Button, badge: &mut frame::Frame, active: bool) {
    if active {
        btn.set_color(col(CLR_RED));
        btn.set_label("STOP");
        badge.set_color(col(CLR_RED));
        badge.set_label("ACTIVE");
    } else {
        btn.set_color(col(CLR_GREEN));
        btn.set_label("START");
        badge.set_color(col(CLR_BADGE_INACTIVE));
        badge.set_label("INACTIVE");
    }
    btn.redraw();
    badge.redraw();
}

/// Attaches title-bar drag-to-move behaviour to the window.
/// Only drags when the push originates in the top bar (y <= 35) and
/// not over the window-control buttons (x < 230).
pub fn attach_drag_handler(wind: &mut window::Window) {
    let mut drag_offset_x = 0;
    let mut drag_offset_y = 0;
    let mut is_dragging = false;

    wind.handle(move |w, ev| match ev {
        Event::Push => {
            let ex = app::event_x();
            let ey = app::event_y();
            if ey >= 0 && ey <= 35 && ex >= 0 && ex < 270 {
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
