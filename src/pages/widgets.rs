use fltk::{button, enums::*, frame, group, prelude::*};
use std::sync::{
    atomic::{AtomicBool, AtomicI32, Ordering},
    Arc, Mutex,
};

use crate::ui::{col, CLR_BADGE_INACTIVE, CLR_GREEN};

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
        assert!(
            !labels.is_empty(),
            "SegmentedControl needs at least one label"
        );
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
                    Color::from_rgb(160, 160, 160)
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
                    Color::from_rgb(160, 160, 160)
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
                Color::from_rgb(160, 160, 160)
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
        slide_tick(slider.clone(), group.clone(), target_idx.clone(), btn_w, seg_h);
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

pub fn build_int_input(x: i32, y: i32, w: i32, h: i32) -> fltk::input::IntInput {
    let mut inp = fltk::input::IntInput::default()
        .with_size(w, h)
        .with_pos(x, y);
    inp.set_color(col(crate::ui::CLR_WIDGET));
    inp.set_text_color(Color::White);
    inp.set_text_font(Font::Helvetica);
    inp.set_text_size(13);
    inp.set_frame(FrameType::RFlatBox);

    inp
}
