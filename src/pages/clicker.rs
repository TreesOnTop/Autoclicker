use crate::pages::widgets::{build_int_input, build_label, SegmentedControl};
use crate::ui::{col, CLR_BADGE_INACTIVE, CLR_GREEN, CLR_GREEN_HOVER, CLR_LABEL, CLR_TITLEBAR};
use fltk::{button, draw, enums::*, frame, group, prelude::*};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

pub struct ClickerHandles {
    pub group: group::Group,
    pub status_badge: frame::Frame,
    pub start_stop_btn: button::Button,

    #[allow(dead_code)]
    pub click_type_btns: [button::Button; 3],

    pub interval_ms: Arc<AtomicI32>,

    pub click_type_index: Arc<AtomicI32>,
}

pub fn build_clicker_page(
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    initial_interval: i32,
    initial_click_type: i32,
    tx: std::sync::mpsc::Sender<()>,
) -> ClickerHandles {
    let clicker_group = group::Group::default().with_size(w, h).with_pos(x, y);

    let mut status_panel = frame::Frame::default()
        .with_size(350, 45)
        .with_pos(x + 20, y + 15);
    status_panel.set_color(col(CLR_TITLEBAR));
    status_panel.set_frame(FrameType::RFlatBox);

    let mut status_title = frame::Frame::default()
        .with_size(80, 45)
        .with_pos(x + 35, y + 15);
    status_title.set_label("Status:");
    status_title.set_label_color(col(CLR_LABEL));
    status_title.set_label_font(Font::HelveticaBold);
    status_title.set_label_size(13);
    status_title.set_align(Align::Left | Align::Inside);

    let mut status_badge = frame::Frame::default()
        .with_size(100, 26)
        .with_pos(x + 255, y + 24);
    status_badge.set_frame(FrameType::RFlatBox);
    status_badge.set_color(col(CLR_BADGE_INACTIVE));
    status_badge.set_label("INACTIVE");
    status_badge.set_label_color(Color::White);
    status_badge.set_label_font(Font::HelveticaBold);
    status_badge.set_label_size(11);

    let _interval_label = build_label("Click Interval (ms)", x + 20, y + 75, 140, 30, 13);

    let mut interval_input = build_int_input(x + 260, y + 75, 110, 30);
    interval_input.set_value(&initial_interval.to_string());

    let _click_type_label = build_label("Click Type", x + 20, y + 125, 140, 30, 13);

    let mut seg = SegmentedControl::new(
        x + 257,
        y + 125,
        114,
        30,
        &["Left", "Mid", "Right"],
        initial_click_type as usize,
    );

    let mut start_stop_btn = button::Button::default()
        .with_size(350, 50)
        .with_pos(x + 20, y + 185);
    start_stop_btn.set_label("START");
    start_stop_btn.set_label_color(Color::White);
    start_stop_btn.set_label_font(Font::HelveticaBold);
    start_stop_btn.set_label_size(16);
    start_stop_btn.set_color(col(CLR_GREEN));
    start_stop_btn.set_selection_color(col(CLR_GREEN_HOVER));
    start_stop_btn.set_frame(FrameType::RFlatBox);
    start_stop_btn.clear_visible_focus();

    start_stop_btn.draw(|b| {
        draw::set_draw_color(b.color());
        draw::draw_rounded_rectf(b.x(), b.y(), b.w(), b.h(), 4);
        draw::set_draw_color(Color::White);
        draw::set_font(b.label_font(), b.label_size());
        draw::draw_text2(&b.label(), b.x(), b.y(), b.w(), b.h(), Align::Center);
    });

    clicker_group.end();

    let interval_ms = Arc::new(AtomicI32::new(initial_interval));
    let click_type_index = Arc::new(AtomicI32::new(initial_click_type));

    let interval_ms_input = interval_ms.clone();
    let interval_input_cb = interval_input.clone();
    let tx_interval = tx.clone();
    interval_input.handle(move |_, ev| {
        if ev == Event::KeyUp || ev == Event::Unfocus {
            let v: i32 = interval_input_cb.value().parse().unwrap_or(100);
            interval_ms_input.store(v.max(1), Ordering::SeqCst);
            let _ = tx_interval.send(());
        }
        false
    });

    let cti0 = click_type_index.clone();
    let tx0 = tx.clone();
    seg.set_callback(0, move |_| {
        cti0.store(0, Ordering::SeqCst);
        let _ = tx0.send(());
    });
    let cti1 = click_type_index.clone();
    let tx1 = tx.clone();
    seg.set_callback(1, move |_| {
        cti1.store(1, Ordering::SeqCst);
        let _ = tx1.send(());
    });
    let cti2 = click_type_index.clone();
    let tx2 = tx.clone();
    seg.set_callback(2, move |_| {
        cti2.store(2, Ordering::SeqCst);
        let _ = tx2.send(());
    });

    let [b0, b1, b2]: [button::Button; 3] =
        seg.buttons.try_into().expect("expected exactly 3 buttons");

    ClickerHandles {
        group: clicker_group,
        status_badge,
        start_stop_btn,
        click_type_btns: [b0, b1, b2],
        interval_ms,
        click_type_index,
    }
}
