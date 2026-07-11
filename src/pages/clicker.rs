use crate::pages::widgets::{DelayPillInput, SegmentedControl, build_int_input, build_label};
use crate::speed::{
    DelayParts, MAX_CPS, MAX_DELAY_HOURS, MIN_INTERVAL_MS, SPEED_MODE_DELAY, SPEED_MODE_RATE,
    SpeedState, clamp_rate_count, interval_from_delay, normalize_rate_unit, normalize_speed_mode,
};
use crate::ui::{
    CLR_BADGE_INACTIVE, CLR_GREEN, CLR_GREEN_HOVER, CLR_LABEL, CLR_TITLEBAR, CLR_WIDGET, col,
};
use fltk::{button, draw, enums::*, frame, group, input, menu, prelude::*};
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

pub struct ClickerHandles {
    pub group: group::Group,
    pub status_badge: frame::Frame,
    pub start_stop_btn: button::Button,
    pub speed: SpeedState,
    pub click_type_index: Arc<AtomicI32>,
}

pub struct ClickerSpeedInitial {
    pub speed_mode: i32,
    pub delay_h: u16,
    pub delay_m: u8,
    pub delay_s: u8,
    pub delay_ms: u16,
    pub rate_count: i32,
    pub rate_unit: i32,
    pub click_type_index: i32,
}

fn canonical_number(input: &mut input::IntInput) -> i64 {
    let raw = input.value();
    let digits = raw.trim_start_matches('0');
    let canonical = if digits.is_empty() { "0" } else { digits };
    if raw != canonical {
        input.set_value(canonical);
    }
    canonical.parse::<i64>().unwrap_or(i64::MAX)
}

pub fn build_clicker_page(
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    initial: ClickerSpeedInitial,
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

    let speed_mode_init = normalize_speed_mode(initial.speed_mode);
    let mut mode_seg = SegmentedControl::new(
        x + 20,
        y + 75,
        350,
        30,
        &["Delay", "Rate"],
        speed_mode_init as usize,
    );

    let delay_y = y + 115;
    let mut delay_pill = DelayPillInput::new(
        x + 20,
        delay_y,
        350,
        36,
        DelayParts {
            hours: initial.delay_h,
            minutes: initial.delay_m,
            seconds: initial.delay_s,
            milliseconds: initial.delay_ms,
        },
    );

    let mut lbl_rate = build_label("Clicks", x + 20, delay_y, 50, 30, 12);
    let mut inp_rate = build_int_input(x + 70, delay_y, 70, 30);
    inp_rate.set_value(
        &clamp_rate_count(initial.rate_count, normalize_rate_unit(initial.rate_unit)).to_string(),
    );

    let mut lbl_per = build_label("per", x + 148, delay_y, 30, 30, 12);
    let mut unit_choice = menu::Choice::default()
        .with_size(170, 30)
        .with_pos(x + 180, delay_y);
    unit_choice.add_choice("Second|Minute|Hour|Day");
    unit_choice.set_value(normalize_rate_unit(initial.rate_unit));
    unit_choice.set_color(col(CLR_WIDGET));
    unit_choice.set_text_color(Color::White);
    unit_choice.set_text_font(Font::Helvetica);
    unit_choice.set_text_size(13);
    unit_choice.set_frame(FrameType::RFlatBox);

    let delay_visible = speed_mode_init == SPEED_MODE_DELAY;
    if delay_visible {
        lbl_rate.hide();
        inp_rate.hide();
        lbl_per.hide();
        unit_choice.hide();
    } else {
        delay_pill.hide();
    }

    let _click_type_label = build_label("Click Type", x + 20, y + 165, 140, 30, 13);

    let mut seg = SegmentedControl::new(
        x + 257,
        y + 165,
        114,
        30,
        &["Left", "Mid", "Right"],
        initial.click_type_index.clamp(0, 2) as usize,
    );

    let mut start_stop_btn = button::Button::default()
        .with_size(350, 50)
        .with_pos(x + 20, y + 225);
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

    let speed = SpeedState::new(
        speed_mode_init,
        initial.delay_h,
        initial.delay_m,
        initial.delay_s,
        initial.delay_ms,
        initial.rate_count,
        initial.rate_unit,
    );
    let click_type_index = Arc::new(AtomicI32::new(initial.click_type_index.clamp(0, 2)));

    if speed_mode_init == SPEED_MODE_DELAY
        && interval_from_delay(
            speed.delay_h.load(Ordering::SeqCst).max(0) as u16,
            speed.delay_m.load(Ordering::SeqCst).max(0) as u8,
            speed.delay_s.load(Ordering::SeqCst).max(0) as u8,
            speed.delay_ms.load(Ordering::SeqCst).max(0) as u16,
        ) == MIN_INTERVAL_MS
    {
        speed.delay_ms.store(MIN_INTERVAL_MS, Ordering::SeqCst);
        delay_pill
            .milliseconds
            .set_value(&MIN_INTERVAL_MS.to_string());
        delay_pill.relayout();
    }

    speed.sync_interval();

    {
        let speed_cb = speed.clone();
        let tx_mode = tx.clone();

        let mut delay_pill_cb = delay_pill.clone();
        let mut lbl_rate_cb = lbl_rate.clone();
        let mut inp_rate_cb = inp_rate.clone();
        let mut lbl_per_cb = lbl_per.clone();
        let mut unit_choice_cb = unit_choice.clone();

        mode_seg.set_callback(0, move |_| {
            speed_cb.mode.store(SPEED_MODE_DELAY, Ordering::SeqCst);
            if interval_from_delay(
                speed_cb.delay_h.load(Ordering::SeqCst).max(0) as u16,
                speed_cb.delay_m.load(Ordering::SeqCst).max(0) as u8,
                speed_cb.delay_s.load(Ordering::SeqCst).max(0) as u8,
                speed_cb.delay_ms.load(Ordering::SeqCst).max(0) as u16,
            ) == MIN_INTERVAL_MS
            {
                speed_cb.delay_ms.store(MIN_INTERVAL_MS, Ordering::SeqCst);
                delay_pill_cb
                    .milliseconds
                    .set_value(&MIN_INTERVAL_MS.to_string());
                delay_pill_cb.relayout();
            }
            delay_pill_cb.show();
            lbl_rate_cb.hide();
            inp_rate_cb.hide();
            lbl_per_cb.hide();
            unit_choice_cb.hide();
            speed_cb.sync_interval();
            let _ = tx_mode.send(());
        });
    }
    {
        let speed_cb = speed.clone();
        let tx_mode = tx.clone();

        let mut delay_pill_cb = delay_pill.clone();
        let mut lbl_rate_cb = lbl_rate.clone();
        let mut inp_rate_cb = inp_rate.clone();
        let mut lbl_per_cb = lbl_per.clone();
        let mut unit_choice_cb = unit_choice.clone();

        mode_seg.set_callback(1, move |_| {
            speed_cb.mode.store(SPEED_MODE_RATE, Ordering::SeqCst);
            delay_pill_cb.hide();
            lbl_rate_cb.show();
            inp_rate_cb.show();
            lbl_per_cb.show();
            unit_choice_cb.show();
            speed_cb.sync_interval();
            let _ = tx_mode.send(());
        });
    }

    fn wire_delay_input(
        input: &mut input::IntInput,
        field_index: usize,
        mut fields: [input::IntInput; 4],
        atomics: [Arc<AtomicI32>; 4],
        mut pill: DelayPillInput,
        speed: SpeedState,
        tx: std::sync::mpsc::Sender<()>,
    ) {
        input.set_readonly(false);
        input.set_trigger(CallbackTrigger::Changed);
        input.set_callback(move |input| {
            let cursor_position = input.position();
            let mut values = fields.iter_mut().map(canonical_number).collect::<Vec<_>>();
            values[field_index] = values[field_index].max(0);

            values[0] = values[0].clamp(0, MAX_DELAY_HOURS as i64);
            for index in (1..4).rev() {
                let base = if index == 3 { 1000 } else { 60 };
                values[index] = values[index].max(0);
                if values[index] >= base {
                    values[index - 1] = values[index - 1].saturating_add(values[index] / base);
                    values[index] %= base;
                }
            }
            values[0] = values[0].clamp(0, MAX_DELAY_HOURS as i64);

            if interval_from_delay(
                values[0] as u16,
                values[1] as u8,
                values[2] as u8,
                values[3] as u16,
            ) == MIN_INTERVAL_MS
            {
                values[3] = MIN_INTERVAL_MS as i64;
            }

            for (index, value) in values.into_iter().enumerate() {
                let numeric_value = value as i32;
                let value = value.to_string();
                if fields[index].value() != value {
                    fields[index].set_value(&value);
                }
                atomics[index].store(numeric_value, Ordering::SeqCst);
            }
            let max_position = fields[field_index].value().len() as i32;
            let restored_position = cursor_position.clamp(0, max_position);
            if fields[field_index].position() != restored_position {
                let _ = fields[field_index].set_position(restored_position);
            }
            pill.relayout();
            speed.sync_interval();
            let _ = tx.send(());
        });
    }

    let delay_inputs = [
        delay_pill.hours.clone(),
        delay_pill.minutes.clone(),
        delay_pill.seconds.clone(),
        delay_pill.milliseconds.clone(),
    ];
    let delay_atomics = [
        speed.delay_h.clone(),
        speed.delay_m.clone(),
        speed.delay_s.clone(),
        speed.delay_ms.clone(),
    ];

    for field_index in 0..delay_inputs.len() {
        let mut input = delay_inputs[field_index].clone();
        wire_delay_input(
            &mut input,
            field_index,
            delay_inputs.clone(),
            delay_atomics.clone(),
            delay_pill.clone(),
            speed.clone(),
            tx.clone(),
        );
    }

    {
        let speed_cb = speed.clone();
        let tx_rate = tx.clone();
        inp_rate.set_trigger(CallbackTrigger::Changed);
        inp_rate.set_callback(move |input| {
            let cursor_position = input.position();
            let v = canonical_number(input).clamp(1, MAX_CPS as i64) as i32;
            let text = v.to_string();
            if input.value() != text {
                input.set_value(&text);
                let max_position = text.len() as i32;
                let restored = cursor_position.clamp(0, max_position);
                if input.position() != restored {
                    let _ = input.set_position(restored);
                }
            }
            speed_cb.rate_count.store(v, Ordering::SeqCst);
            speed_cb.sync_interval();
            let _ = tx_rate.send(());
        });
    }

    {
        let speed_cb = speed.clone();
        let tx_unit = tx.clone();
        let mut inp_rate_cb = inp_rate.clone();
        unit_choice.set_callback(move |c| {
            let unit = normalize_rate_unit(c.value());
            speed_cb.rate_unit.store(unit, Ordering::SeqCst);
            let count = clamp_rate_count(speed_cb.rate_count.load(Ordering::SeqCst), unit);
            speed_cb.rate_count.store(count, Ordering::SeqCst);
            inp_rate_cb.set_value(&count.to_string());
            speed_cb.sync_interval();
            let _ = tx_unit.send(());
        });
    }

    for index in 0..3 {
        let click_type = click_type_index.clone();
        let tx_click_type = tx.clone();
        seg.set_callback(index, move |selected| {
            click_type.store(selected as i32, Ordering::SeqCst);
            let _ = tx_click_type.send(());
        });
    }

    ClickerHandles {
        group: clicker_group,
        status_badge,
        start_stop_btn,
        speed,
        click_type_index,
    }
}
