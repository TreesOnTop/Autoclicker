use crate::pages::widgets::{build_label, SegmentedControl};
use crate::settings_io::ProcessEntry;
use crate::ui::{col, CLR_BADGE_INACTIVE, CLR_GREEN, CLR_LABEL, CLR_RED, CLR_WIDGET};
use fltk::{
    button,
    enums::*,
    frame,
    group::{self, Scroll, ScrollType},
    prelude::*,
};
use std::sync::{Arc, Mutex};

pub struct ProcessesHandles {
    pub group: group::Group,
    pub filter_mode_control: SegmentedControl,
    pub ui_entries: Arc<Mutex<Vec<ProcessEntry>>>,
    pub row_pack: group::Pack,
    pub scroll: Scroll,
}

const ROW_H: i32 = 30;
const ROW_PAD: i32 = 6;
const CHECK_W: i32 = 20;
const ACTION_CTRL_W: i32 = 86;
const ACTION_CTRL_H: i32 = 22;

pub fn build_processes_page(
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    initial_filter_mode: i32,
    initial_processes: Vec<ProcessEntry>,
    tx: std::sync::mpsc::Sender<()>,
) -> ProcessesHandles {
    let mut processes_group = group::Group::default().with_size(w, h).with_pos(x, y);

    let _mode_label = build_label("Filter", x + 10, y + 15, 45, 26, 12);

    let mut filter_mode_control = SegmentedControl::new(
        x + 60,
        y + 15,
        150,
        26,
        &["Whitelist", "Blacklist"],
        initial_filter_mode as usize,
    );

    let header_y = y + 50;
    let scroll_y = header_y + 22;
    let scroll_h = h - (scroll_y - y) - 4;
    let scroll_x = x + 10;
    let scroll_w = w - 20;

    let mut hdr_proc = frame::Frame::default()
        .with_size(scroll_w - ACTION_CTRL_W - ROW_PAD * 2, 18)
        .with_pos(scroll_x + ROW_PAD + CHECK_W + ROW_PAD, header_y + 2);
    hdr_proc.set_label("Process");
    hdr_proc.set_label_color(col(CLR_LABEL));
    hdr_proc.set_label_font(Font::Helvetica);
    hdr_proc.set_label_size(11);
    hdr_proc.set_align(Align::Left | Align::Inside);

    let action_hdr_x = scroll_x + scroll_w - ACTION_CTRL_W - ROW_PAD;
    let mut hdr_action = frame::Frame::default()
        .with_size(ACTION_CTRL_W, 18)
        .with_pos(action_hdr_x, header_y + 2);
    hdr_action.set_label("Action");
    hdr_action.set_label_color(col(CLR_LABEL));
    hdr_action.set_label_font(Font::Helvetica);
    hdr_action.set_label_size(11);
    hdr_action.set_align(Align::Center | Align::Inside);
    let show_actions = initial_filter_mode == 1;
    if !show_actions {
        hdr_action.hide();
    }

    let mut scroll = Scroll::default()
        .with_size(scroll_w, scroll_h)
        .with_pos(scroll_x, scroll_y);
    scroll.set_type(ScrollType::Vertical);
    scroll.set_color(col(CLR_WIDGET));
    scroll.set_frame(FrameType::RFlatBox);

    let pack_w = scroll_w - 17;
    let mut row_pack = group::Pack::default()
        .with_size(pack_w, 0)
        .with_pos(scroll_x, scroll_y);
    row_pack.set_type(group::PackType::Vertical);
    row_pack.set_spacing(1);

    row_pack.end();
    scroll.end();

    processes_group.end();
    processes_group.hide();

    let ui_entries: Arc<Mutex<Vec<ProcessEntry>>> = Arc::new(Mutex::new(Vec::new()));

    let open_procs = crate::platform::get_open_processes();
    let initial_rows = build_initial_entries(&open_procs, &initial_processes);

    {
        let mut guard = ui_entries.lock().unwrap();
        *guard = initial_rows.clone();
    }

    populate_rows(
        &mut row_pack,
        &scroll,
        &initial_rows,
        pack_w,
        &ui_entries,
        show_actions,
        tx.clone(),
    );

    let tx_choice1 = tx.clone();
    let mut hdr_action_wl = hdr_action.clone();
    let mut row_pack_wl = row_pack.clone();
    let scroll_wl = scroll.clone();
    let ui_entries_wl = ui_entries.clone();
    let pack_w_wl = pack_w;
    filter_mode_control.set_callback(0, move |_| {
        hdr_action_wl.hide();
        let entries = ui_entries_wl.lock().unwrap().clone();
        populate_rows(
            &mut row_pack_wl,
            &scroll_wl,
            &entries,
            pack_w_wl,
            &ui_entries_wl,
            false,
            tx_choice1.clone(),
        );
        let _ = tx_choice1.send(());
    });

    let tx_choice0 = tx.clone();
    let mut hdr_action_bl = hdr_action.clone();
    let mut row_pack_bl = row_pack.clone();
    let scroll_bl = scroll.clone();
    let ui_entries_bl = ui_entries.clone();
    let pack_w_bl = pack_w;
    filter_mode_control.set_callback(1, move |_| {
        hdr_action_bl.show();
        let entries = ui_entries_bl.lock().unwrap().clone();
        populate_rows(
            &mut row_pack_bl,
            &scroll_bl,
            &entries,
            pack_w_bl,
            &ui_entries_bl,
            true,
            tx_choice0.clone(),
        );
        let _ = tx_choice0.send(());
    });

    ProcessesHandles {
        group: processes_group,
        filter_mode_control,
        ui_entries,
        row_pack,
        scroll,
    }
}


fn build_initial_entries(
    open_procs: &[String],
    saved: &[ProcessEntry],
) -> Vec<ProcessEntry> {
    open_procs
        .iter()
        .map(|proc_name| {
            if let Some(saved_entry) = saved.iter().find(|e| &e.name == proc_name) {
                ProcessEntry::new(proc_name.clone(), saved_entry.action, saved_entry.enabled)
            } else {
                ProcessEntry::new(proc_name.clone(), 1, false)
            }
        })
        .collect()
}

pub fn refresh_process_rows(
    row_pack: &mut group::Pack,
    scroll: &Scroll,
    ui_entries: &Arc<Mutex<Vec<ProcessEntry>>>,
    pack_w: i32,
    show_actions: bool,
    tx: std::sync::mpsc::Sender<()>,
) {
    let open_procs = crate::platform::get_open_processes();
    let current_entries = ui_entries.lock().unwrap().clone();
    let new_entries = build_initial_entries(&open_procs, &current_entries);

    {
        let mut guard = ui_entries.lock().unwrap();
        *guard = new_entries.clone();
    }

    populate_rows(
        row_pack,
        scroll,
        &new_entries,
        pack_w,
        ui_entries,
        show_actions,
        tx,
    );
}

fn populate_rows(
    row_pack: &mut group::Pack,
    scroll: &Scroll,
    entries: &[ProcessEntry],
    pack_w: i32,
    ui_entries: &Arc<Mutex<Vec<ProcessEntry>>>,
    show_actions: bool,
    tx: std::sync::mpsc::Sender<()>,
) {
    row_pack.clear();
    row_pack.begin();

    for (i, entry) in entries.iter().enumerate() {
        build_row(i, entry, pack_w, ui_entries, show_actions, tx.clone());
    }

    row_pack.end();

    let total_h = entries.len() as i32 * (ROW_H + 1);
    let px = row_pack.x();
    let py = row_pack.y();
    row_pack.resize(px, py, pack_w, total_h.max(1));

    if let Some(mut p) = scroll.parent() {
        p.redraw();
    } else {
        let mut s = scroll.clone();
        s.redraw();
    }
}

fn build_row(
    index: usize,
    entry: &ProcessEntry,
    row_w: i32,
    ui_entries: &Arc<Mutex<Vec<ProcessEntry>>>,
    show_actions: bool,
    tx: std::sync::mpsc::Sender<()>,
) {
    let action_space = if show_actions {
        ACTION_CTRL_W + ROW_PAD
    } else {
        0
    };
    let label_w = row_w - ROW_PAD - CHECK_W - ROW_PAD - action_space;

    let mut row_grp = group::Group::default().with_size(row_w, ROW_H);
    row_grp.set_frame(FrameType::FlatBox);
    if index % 2 == 0 {
        row_grp.set_color(col(CLR_WIDGET));
    } else {
        row_grp.set_color(Color::from_rgb(25, 25, 25));
    }

    let rx = row_grp.x();
    let ry = row_grp.y();

    let check_x = rx + ROW_PAD;
    let check_y = ry + (ROW_H - CHECK_W) / 2;
    let mut chk = button::CheckButton::default()
        .with_size(CHECK_W, CHECK_W)
        .with_pos(check_x, check_y);
    chk.set_value(entry.enabled);
    chk.set_frame(FrameType::RFlatBox);
    chk.set_color(col(CLR_BADGE_INACTIVE));
    chk.set_selection_color(col(CLR_GREEN));
    chk.clear_visible_focus();

    let lbl_x = check_x + CHECK_W + ROW_PAD;
    let mut lbl = frame::Frame::default()
        .with_size(label_w, ROW_H)
        .with_pos(lbl_x, ry);
    lbl.set_label(&entry.name);
    lbl.set_label_color(Color::White);
    lbl.set_label_font(Font::Helvetica);
    lbl.set_label_size(12);
    lbl.set_align(Align::Left | Align::Inside);

    let action_x = rx + row_w - ACTION_CTRL_W - ROW_PAD;
    let action_y = ry + (ROW_H - ACTION_CTRL_H) / 2;
    let mut action_ctrl = SegmentedControl::new(
        action_x,
        action_y,
        ACTION_CTRL_W,
        ACTION_CTRL_H,
        &["Stop", "Pause"],
        entry.action.clamp(0, 1) as usize,
    );
    action_ctrl.set_segment_colors(&[col(CLR_RED), col(CLR_GREEN)]);
    action_ctrl.set_enabled(entry.enabled);
    if !show_actions {
        action_ctrl.hide();
    }

    row_grp.end();


    let entries_cb = ui_entries.clone();
    let tx_chk = tx.clone();
    let mut action_for_chk = action_ctrl.clone();
    chk.set_callback(move |b| {
        let enabled = b.value();
        if let Ok(mut guard) = entries_cb.lock() {
            if let Some(e) = guard.get_mut(index) {
                e.enabled = enabled;
            }
        }
        action_for_chk.set_enabled(enabled);
        let _ = tx_chk.send(());
    });

    let entries_stop = ui_entries.clone();
    let tx_stop = tx.clone();
    action_ctrl.set_callback(0, move |_| {
        if let Ok(mut guard) = entries_stop.lock() {
            if let Some(e) = guard.get_mut(index) {
                e.action = 0;
            }
        }
        let _ = tx_stop.send(());
    });

    let entries_pause = ui_entries.clone();
    let tx_pause = tx.clone();
    action_ctrl.set_callback(1, move |_| {
        if let Ok(mut guard) = entries_pause.lock() {
            if let Some(e) = guard.get_mut(index) {
                e.action = 1;
            }
        }
        let _ = tx_pause.send(());
    });
}

