use crate::pages::widgets::{build_action_button, build_input, build_label};
use crate::ui::{col, CLR_GREEN, CLR_GREEN_HOVER, CLR_RED, CLR_RED_HOVER, CLR_WIDGET};
use fltk::{browser, enums::*, group, menu, prelude::*};

pub struct ProcessesHandles {
    pub group: group::Group,
    pub filter_mode_choice: menu::Choice,
    pub processes_browser: browser::HoldBrowser,
}

pub fn build_processes_page(
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    initial_filter_mode: i32,
    initial_processes: Vec<String>,
    tx: std::sync::mpsc::Sender<()>,
) -> ProcessesHandles {
    let mut processes_group = group::Group::default().with_size(w, h).with_pos(x, y);

    let _mode_label = build_label("Filter Mode", x + 20, y + 15, 100, 30, 13);

    let mut mode_choice = menu::Choice::default()
        .with_size(210, 30)
        .with_pos(x + 160, y + 15);
    mode_choice.set_color(col(CLR_WIDGET));
    mode_choice.set_text_color(Color::White);
    mode_choice.set_text_font(Font::Helvetica);
    mode_choice.set_text_size(13);
    mode_choice.set_frame(FrameType::RFlatBox);
    mode_choice.add_choice("Blacklist (Disable click)");
    mode_choice.add_choice("Whitelist (Enable click)");
    mode_choice.set_value(initial_filter_mode);
    mode_choice.clear_visible_focus();

    let proc_input = build_input(x + 20, y + 60, 260, 30);

    let mut add_btn = build_action_button(
        "Add",
        x + 290,
        y + 60,
        80,
        30,
        col(CLR_GREEN),
        col(CLR_GREEN_HOVER),
    );

    let mut list_browser = browser::HoldBrowser::default()
        .with_size(260, 140)
        .with_pos(x + 20, y + 105);
    list_browser.set_color(col(CLR_WIDGET));
    list_browser.set_text_size(13);
    list_browser.set_frame(FrameType::RFlatBox);
    list_browser.clear_visible_focus();
    list_browser.clear();
    for proc in initial_processes {
        list_browser.add(&proc);
    }

    let mut remove_btn = build_action_button(
        "Remove",
        x + 290,
        y + 105,
        80,
        30,
        col(CLR_RED),
        col(CLR_RED_HOVER),
    );

    processes_group.end();
    processes_group.hide();

    let mut browser_clone = list_browser.clone();
    let input_clone = proc_input.clone();
    let tx_add = tx.clone();
    add_btn.set_callback(move |_| {
        let val = input_clone.value().trim().to_string();
        if !val.is_empty() {
            browser_clone.add(&val);
            let mut input_mut = input_clone.clone();
            input_mut.set_value("");
            let _ = tx_add.send(());
        }
    });

    let mut browser_clone2 = list_browser.clone();
    let tx_remove = tx.clone();
    remove_btn.set_callback(move |_| {
        let selected = browser_clone2.value();
        if selected > 0 {
            browser_clone2.remove(selected);
            let _ = tx_remove.send(());
        }
    });

    let tx_choice = tx.clone();
    mode_choice.set_callback(move |_| {
        let _ = tx_choice.send(());
    });

    ProcessesHandles {
        group: processes_group,
        filter_mode_choice: mode_choice,
        processes_browser: list_browser,
    }
}
