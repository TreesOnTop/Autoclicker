use crate::speed::{DelayParts, MAX_DELAY_HOURS};
use crate::ui::{CLR_BADGE_INACTIVE, CLR_GREEN, CLR_SEGMENT_INACTIVE, CLR_WIDGET, col};
use fltk::{button, enums::*, frame, group, input::IntInput, prelude::*};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering},
};

fn enable_caret_blink(inp: &mut IntInput) {
    let visible = Color::White;
    let generation = Arc::new(AtomicU64::new(0));
    let caret_on = Arc::new(AtomicBool::new(true));

    let restart = {
        let generation = generation.clone();
        let caret_on = caret_on.clone();
        move |w: &mut IntInput| {
            let tick = generation.fetch_add(1, Ordering::SeqCst) + 1;
            caret_on.store(true, Ordering::SeqCst);
            w.set_cursor_color(visible);
            if crate::platform::get_caret_blink_secs().is_some() {
                schedule_caret_blink(
                    w.clone(),
                    visible,
                    generation.clone(),
                    tick,
                    caret_on.clone(),
                );
            }
        }
    };

    let generation_h = generation.clone();
    let caret_on_h = caret_on.clone();
    inp.handle(move |w, ev| match ev {
        Event::Focus => {
            restart(w);
            false
        }
        Event::Unfocus => {
            generation_h.fetch_add(1, Ordering::SeqCst);
            caret_on_h.store(true, Ordering::SeqCst);
            w.set_cursor_color(visible);
            false
        }
        Event::KeyDown | Event::Paste | Event::Push => {
            restart(w);
            false
        }
        _ => false,
    });
}

fn schedule_caret_blink(
    mut inp: IntInput,
    visible: Color,
    generation: Arc<AtomicU64>,
    my_tick: u64,
    caret_on: Arc<AtomicBool>,
) {
    let Some(delay) = crate::platform::get_caret_blink_secs() else {
        return;
    };
    fltk::app::add_timeout3(delay, move |_| {
        if generation.load(Ordering::SeqCst) != my_tick || !inp.has_focus() {
            return;
        }
        let show = !caret_on.load(Ordering::SeqCst);
        caret_on.store(show, Ordering::SeqCst);
        inp.set_cursor_color(if show { visible } else { inp.color() });
        inp.redraw();
        schedule_caret_blink(
            inp.clone(),
            visible,
            generation.clone(),
            my_tick,
            caret_on.clone(),
        );
    });
}
#[derive(Clone)]
pub struct SegmentedControl {
    pub group: group::Group,

    pub buttons: Vec<button::Button>,

    pub btn_w: i32,

    seg_h: i32,

    slider: frame::Frame,

    target_idx: Arc<AtomicI32>,

    segment_colors: Arc<Mutex<Vec<Color>>>,

    enabled: Arc<AtomicBool>,
}

impl SegmentedControl {
    pub fn new(x: i32, y: i32, w: i32, h: i32, labels: &[&str], active: usize) -> Self {
        let n = labels.len() as i32;
        let btn_w = w / n;
        let container_clr = col(CLR_BADGE_INACTIVE);
        let active = active.min(labels.len().saturating_sub(1));

        let mut seg_group = group::Group::default().with_size(w, h).with_pos(x, y);
        seg_group.set_frame(FrameType::RFlatBox);
        seg_group.set_color(container_clr);

        let init_slider_x = x + btn_w * active as i32;
        let mut slider = frame::Frame::default()
            .with_size(btn_w, h)
            .with_pos(init_slider_x, y);
        slider.set_frame(FrameType::RFlatBox);
        slider.set_color(col(CLR_GREEN));

        let buttons: Vec<button::Button> = labels
            .iter()
            .enumerate()
            .map(|(i, &lbl)| {
                let mut b = button::Button::default()
                    .with_size(btn_w, h)
                    .with_pos(x + btn_w * i as i32, y);
                b.set_label(lbl);
                let lc = if i == active {
                    Color::White
                } else {
                    col(CLR_SEGMENT_INACTIVE)
                };
                b.set_label_color(lc);
                b.set_label_font(Font::Helvetica);
                b.set_label_size(12);
                b.set_frame(FrameType::NoBox);
                b.set_color(container_clr);
                b.set_selection_color(container_clr);
                b.clear_visible_focus();
                b
            })
            .collect();

        seg_group.end();

        let segment_colors = Arc::new(Mutex::new(vec![col(CLR_GREEN); labels.len()]));
        let enabled = Arc::new(AtomicBool::new(true));
        let target_idx = Arc::new(AtomicI32::new(active as i32));

        let mut ctrl = Self {
            group: seg_group,
            buttons,
            btn_w,
            seg_h: h,
            slider,
            target_idx,
            segment_colors,
            enabled,
        };

        ctrl.wire_internal_callbacks();
        ctrl
    }

    pub fn set_segment_colors(&mut self, colors: &[Color]) {
        if let Ok(mut guard) = self.segment_colors.lock() {
            for (i, c) in colors.iter().enumerate() {
                if i < guard.len() {
                    guard[i] = *c;
                }
            }
        }
        self.refresh_appearance();
    }

    fn wire_internal_callbacks(&mut self) {
        let n = self.buttons.len();
        for active_idx in 0..n {
            self.install_callback(active_idx, None);
        }
    }

    pub fn set_callback<F>(&mut self, index: usize, user_cb: F)
    where
        F: FnMut(usize) + 'static,
    {
        self.install_callback(index, Some(Box::new(user_cb)));
    }

    fn install_callback(&mut self, index: usize, user_cb: Option<Box<dyn FnMut(usize)>>) {
        if index >= self.buttons.len() {
            return;
        }

        let mut clones: Vec<button::Button> = self.buttons.clone();
        let target_idx = self.target_idx.clone();
        let mut slider = self.slider.clone();
        let group = self.group.clone();
        let btn_w = self.btn_w;
        let seg_h = self.seg_h;
        let segment_colors = self.segment_colors.clone();
        let enabled = self.enabled.clone();
        let mut user_cb = user_cb;

        self.buttons[index].set_callback(move |_| {
            if !enabled.load(Ordering::SeqCst) {
                return;
            }

            for (j, b) in clones.iter_mut().enumerate() {
                let lc = if j == index {
                    Color::White
                } else {
                    col(CLR_SEGMENT_INACTIVE)
                };
                b.set_label_color(lc);
                b.redraw();
            }

            target_idx.store(index as i32, Ordering::SeqCst);

            let active_color = segment_colors
                .lock()
                .ok()
                .and_then(|g| g.get(index).copied())
                .unwrap_or_else(|| col(CLR_GREEN));
            slider.set_color(active_color);
            slider.redraw();

            let sl = slider.clone();
            let grp = group.clone();
            let idx = target_idx.clone();
            fltk::app::add_timeout3(0.0, move |_| {
                slide_tick(sl.clone(), grp.clone(), idx.clone(), btn_w, seg_h);
            });

            if let Some(cb) = user_cb.as_mut() {
                cb(index);
            }
        });
    }

    pub fn value(&self) -> i32 {
        self.target_idx.load(Ordering::SeqCst)
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled.store(enabled, Ordering::SeqCst);
        self.refresh_appearance();
    }

    pub fn hide(&mut self) {
        self.group.hide();
    }

    fn refresh_appearance(&mut self) {
        let active = self.value().max(0) as usize;
        let enabled = self.enabled.load(Ordering::SeqCst);
        let active_color = self
            .segment_colors
            .lock()
            .ok()
            .and_then(|g| g.get(active).copied())
            .unwrap_or_else(|| col(CLR_GREEN));

        self.slider.set_color(if enabled {
            active_color
        } else {
            col(CLR_BADGE_INACTIVE)
        });

        for (j, b) in self.buttons.iter_mut().enumerate() {
            let lc = if enabled && j == active {
                Color::White
            } else {
                col(CLR_SEGMENT_INACTIVE)
            };
            b.set_label_color(lc);
            b.redraw();
        }

        self.slider.redraw();
        if let Some(mut p) = self.group.parent() {
            p.redraw();
        } else {
            self.group.redraw();
        }
    }
}

pub fn slide_tick(
    mut slider: frame::Frame,
    group: group::Group,
    target_idx: Arc<AtomicI32>,
    btn_w: i32,
    seg_h: i32,
) {
    let idx = target_idx.load(Ordering::SeqCst).max(0);
    let tx = group.x() + btn_w * idx;
    let ty = group.y();
    let cx = slider.x();
    let diff = tx - cx;

    if diff.abs() <= 1 {
        slider.resize(tx, ty, btn_w, seg_h);
        if let Some(mut p) = slider.parent() {
            p.redraw();
        }
        return;
    }

    let refresh_rate = crate::platform::get_monitor_refresh_rate() as f64;
    let frame_time = 1.0 / refresh_rate;
    let dt_ratio = frame_time * 60.0;
    let step_factor = 1.0 - 0.65f64.powf(dt_ratio);

    let step = ((diff as f64) * step_factor).round() as i32;
    let step = if step == 0 { diff.signum() } else { step };
    slider.resize(cx + step, ty, btn_w, seg_h);
    if let Some(mut p) = slider.parent() {
        p.redraw();
    }

    fltk::app::add_timeout3(frame_time, move |_| {
        slide_tick(
            slider.clone(),
            group.clone(),
            target_idx.clone(),
            btn_w,
            seg_h,
        );
    });
}

pub fn build_label(text: &str, x: i32, y: i32, w: i32, h: i32, size: i32) -> frame::Frame {
    let mut lbl = frame::Frame::default().with_size(w, h).with_pos(x, y);
    lbl.set_label(text);
    lbl.set_label_color(col(crate::ui::CLR_LABEL));
    lbl.set_label_font(Font::Helvetica);
    lbl.set_label_size(size);
    lbl.set_align(Align::Left | Align::Inside);
    lbl
}

pub fn build_title(text: &str, x: i32, y: i32, w: i32, h: i32, size: i32) -> frame::Frame {
    let mut lbl = frame::Frame::default().with_size(w, h).with_pos(x, y);
    lbl.set_label(text);
    lbl.set_label_color(Color::White);
    lbl.set_label_font(Font::HelveticaBold);
    lbl.set_label_size(size);
    lbl.set_align(Align::Left | Align::Inside);
    lbl
}

pub fn build_int_input(x: i32, y: i32, w: i32, h: i32) -> IntInput {
    let mut inp = IntInput::default().with_size(w, h).with_pos(x, y);
    inp.set_color(col(CLR_WIDGET));
    inp.set_text_color(Color::White);
    inp.set_cursor_color(Color::White);
    inp.set_text_font(Font::Helvetica);
    inp.set_text_size(13);
    inp.set_frame(FrameType::RFlatBox);
    enable_caret_blink(&mut inp);

    inp
}

#[derive(Clone)]
pub struct DelayPillInput {
    bg: frame::Frame,
    pub hours: fltk::input::IntInput,
    lbl_h: frame::Frame,
    pub minutes: fltk::input::IntInput,
    lbl_m: frame::Frame,
    pub seconds: fltk::input::IntInput,
    lbl_s: frame::Frame,
    pub milliseconds: fltk::input::IntInput,
    lbl_ms: frame::Frame,
    row_x: i32,
    row_w: i32,
    pill_y: i32,
    pill_h: i32,
}

const PILL_FONT_SIZE: i32 = 14;
const PILL_UNIT_SIZE: i32 = 12;
const PILL_PAD_X: i32 = 16;
const PILL_GAP: i32 = 0;
const PILL_INP_PAD: i32 = 6;

impl DelayPillInput {
    pub fn new(x: i32, y: i32, w: i32, h: i32, initial: DelayParts) -> Self {
        use fltk::draw;
        use fltk::input::IntInput;

        let pill_clr = col(crate::ui::CLR_WIDGET);
        let unit_clr = Color::from_rgb(140, 140, 140);

        let mut bg = frame::Frame::default().with_size(1, h).with_pos(x, y);
        bg.set_frame(FrameType::NoBox);
        bg.set_color(pill_clr);
        bg.draw(move |f| {
            draw::set_draw_color(f.color());
            let radius = f.h() / 2;
            draw::draw_rounded_rectf(f.x(), f.y(), f.w(), f.h(), radius);
        });
        bg.deactivate();

        let inp_h = h - 8;
        let inp_y = y + 4;

        let mut hours_inp = IntInput::default().with_size(24, inp_h).with_pos(x, inp_y);
        style_pill_int_input(&mut hours_inp, pill_clr);
        hours_inp.set_value(&initial.hours.min(MAX_DELAY_HOURS).to_string());

        let mut lbl_h = frame::Frame::default()
            .with_size(16, inp_h)
            .with_pos(x, inp_y);
        style_unit_label(&mut lbl_h, "h", unit_clr);

        let mut minutes_inp = IntInput::default().with_size(24, inp_h).with_pos(x, inp_y);
        style_pill_int_input(&mut minutes_inp, pill_clr);
        minutes_inp.set_value(&initial.minutes.min(59).to_string());

        let mut lbl_m = frame::Frame::default()
            .with_size(16, inp_h)
            .with_pos(x, inp_y);
        style_unit_label(&mut lbl_m, "m", unit_clr);

        let mut seconds_inp = IntInput::default().with_size(24, inp_h).with_pos(x, inp_y);
        style_pill_int_input(&mut seconds_inp, pill_clr);
        seconds_inp.set_value(&initial.seconds.min(59).to_string());

        let mut lbl_s = frame::Frame::default()
            .with_size(16, inp_h)
            .with_pos(x, inp_y);
        style_unit_label(&mut lbl_s, "s", unit_clr);

        let mut ms_inp = IntInput::default().with_size(24, inp_h).with_pos(x, inp_y);
        style_pill_int_input(&mut ms_inp, pill_clr);
        ms_inp.set_value(&initial.milliseconds.min(999).to_string());

        let mut lbl_ms = frame::Frame::default()
            .with_size(24, inp_h)
            .with_pos(x, inp_y);
        style_unit_label(&mut lbl_ms, "ms", unit_clr);

        let mut pill = Self {
            bg,
            hours: hours_inp,
            lbl_h,
            minutes: minutes_inp,
            lbl_m,
            seconds: seconds_inp,
            lbl_s,
            milliseconds: ms_inp,
            lbl_ms,
            row_x: x,
            row_w: w,
            pill_y: y,
            pill_h: h,
        };
        pill.relayout();
        pill
    }

    pub fn hide(&mut self) {
        self.bg.hide();
        self.hours.hide();
        self.lbl_h.hide();
        self.minutes.hide();
        self.lbl_m.hide();
        self.seconds.hide();
        self.lbl_s.hide();
        self.milliseconds.hide();
        self.lbl_ms.hide();
    }

    pub fn show(&mut self) {
        self.bg.show();
        self.hours.show();
        self.lbl_h.show();
        self.minutes.show();
        self.lbl_m.show();
        self.seconds.show();
        self.lbl_s.show();
        self.milliseconds.show();
        self.lbl_ms.show();
        self.bg.deactivate();
    }

    pub fn relayout(&mut self) {
        use fltk::draw;

        clamp_if_over(&mut self.hours, MAX_DELAY_HOURS as i32);
        clamp_if_over(&mut self.minutes, 59);
        clamp_if_over(&mut self.seconds, 59);
        clamp_if_over(&mut self.milliseconds, 999);

        draw::set_font(Font::HelveticaBold, PILL_FONT_SIZE);
        let h_w = measure_input_width(&self.hours.value());
        let m_w = measure_input_width(&self.minutes.value());
        let s_w = measure_input_width(&self.seconds.value());
        let ms_w = measure_input_width(&self.milliseconds.value());

        draw::set_font(Font::Helvetica, PILL_UNIT_SIZE);
        let unit_w =
            measure_label_width("h").max(measure_label_width("m").max(measure_label_width("s")));
        let ms_unit_w = measure_label_width("ms");

        let inp_h = self.pill_h - 8;
        let inp_y = self.pill_y + 4;
        let total_w = PILL_PAD_X
            + h_w
            + unit_w
            + PILL_GAP
            + m_w
            + unit_w
            + PILL_GAP
            + s_w
            + unit_w
            + PILL_GAP
            + ms_w
            + ms_unit_w
            + PILL_PAD_X;

        let pill_x = self.row_x + ((self.row_w - total_w) / 2).max(0);
        let old_x = self.bg.x();
        let old_w = self.bg.w();

        place(&mut self.bg, pill_x, self.pill_y, total_w, self.pill_h);
        self.bg.deactivate();

        let mut cursor = pill_x + PILL_PAD_X;
        place(&mut self.hours, cursor, inp_y, h_w, inp_h);
        cursor += h_w;
        place(&mut self.lbl_h, cursor, inp_y, unit_w, inp_h);
        cursor += unit_w + PILL_GAP;
        place(&mut self.minutes, cursor, inp_y, m_w, inp_h);
        cursor += m_w;
        place(&mut self.lbl_m, cursor, inp_y, unit_w, inp_h);
        cursor += unit_w + PILL_GAP;
        place(&mut self.seconds, cursor, inp_y, s_w, inp_h);
        cursor += s_w;
        place(&mut self.lbl_s, cursor, inp_y, unit_w, inp_h);
        cursor += unit_w + PILL_GAP;
        place(&mut self.milliseconds, cursor, inp_y, ms_w, inp_h);
        cursor += ms_w;
        place(&mut self.lbl_ms, cursor, inp_y, ms_unit_w, inp_h);
        let _ = cursor;

        self.bg.redraw();
        self.hours.redraw();
        self.minutes.redraw();
        self.seconds.redraw();
        self.milliseconds.redraw();
        self.lbl_h.redraw();
        self.lbl_m.redraw();
        self.lbl_s.redraw();
        self.lbl_ms.redraw();

        if old_x != pill_x || old_w != total_w {
            if let Some(mut parent) = self.hours.parent() {
                parent.redraw();
            }
            if let Some(mut window) = self.bg.window() {
                window.redraw();
            }
        }
    }
}

fn place(w: &mut impl WidgetExt, x: i32, y: i32, width: i32, height: i32) {
    if w.x() != x || w.y() != y || w.w() != width || w.h() != height {
        w.resize(x, y, width, height);
    }
}

fn measure_input_width(text: &str) -> i32 {
    use fltk::draw;
    let sample = if text.is_empty() { "0" } else { text };
    draw::set_font(Font::HelveticaBold, PILL_FONT_SIZE);
    let (tw, _) = draw::measure(sample, false);
    (tw + PILL_INP_PAD * 2).max(20)
}

fn measure_label_width(text: &str) -> i32 {
    use fltk::draw;
    draw::set_font(Font::Helvetica, PILL_UNIT_SIZE);
    let (tw, _) = draw::measure(text, false);
    tw + 4
}

fn clamp_if_over(inp: &mut fltk::input::IntInput, max: i32) {
    let raw = inp.value();
    if let Ok(v) = raw.parse::<i32>()
        && v > max
    {
        inp.set_value(&max.to_string());
    }
}

fn style_pill_int_input(inp: &mut IntInput, bg: Color) {
    inp.set_frame(FrameType::FlatBox);
    inp.set_color(bg);
    inp.set_selection_color(Color::from_rgb(70, 70, 75));
    inp.set_text_color(Color::White);
    inp.set_cursor_color(Color::White);
    inp.set_text_font(Font::HelveticaBold);
    inp.set_text_size(PILL_FONT_SIZE);
    inp.activate();
    enable_caret_blink(inp);
}

fn style_unit_label(lbl: &mut frame::Frame, text: &str, color: Color) {
    use fltk::draw;
    let label = text.to_string();
    lbl.set_frame(FrameType::NoBox);
    lbl.deactivate();
    lbl.draw(move |f| {
        draw::set_draw_color(color);
        draw::set_font(Font::Helvetica, PILL_UNIT_SIZE);
        draw::draw_text2(
            &label,
            f.x(),
            f.y(),
            f.w(),
            f.h(),
            Align::Left | Align::Inside,
        );
    });
}
