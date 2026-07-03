use fltk::{button, enums::*, frame, group, prelude::*};
use std::sync::{
    atomic::{AtomicI32, Ordering},
    Arc,
};

use crate::ui::{col, CLR_BADGE_INACTIVE, CLR_GREEN};

pub struct SegmentedControl {
    #[allow(dead_code)]
    pub group: group::Group,

    pub buttons: Vec<button::Button>,

    pub btn_w: i32,

    seg_x: i32,

    seg_y: i32,

    seg_h: i32,

    slider: frame::Frame,

    target_x: Arc<AtomicI32>,
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

        let target_x = Arc::new(AtomicI32::new(init_slider_x));

        let mut ctrl = Self {
            group: seg_group,
            buttons,
            btn_w,
            seg_x: x,
            seg_y: y,
            seg_h: h,
            slider,
            target_x,
        };

        ctrl.wire_internal_callbacks();
        ctrl
    }

    fn wire_internal_callbacks(&mut self) {
        let n = self.buttons.len();
        for active_idx in 0..n {
            let mut clones: Vec<button::Button> = self.buttons.clone();
            let target_x_clone = self.target_x.clone();
            let slider_clone = self.slider.clone();
            let dest_x = self.seg_x + self.btn_w * active_idx as i32;
            let seg_y = self.seg_y;
            let btn_w = self.btn_w;
            let seg_h = self.seg_h;

            self.buttons[active_idx].set_callback(move |_| {
                for (j, b) in clones.iter_mut().enumerate() {
                    let lc = if j == active_idx {
                        Color::White
                    } else {
                        Color::from_rgb(160, 160, 160)
                    };
                    b.set_label_color(lc);
                    b.redraw();
                }

                target_x_clone.store(dest_x, Ordering::SeqCst);
                let sl = slider_clone.clone();
                let tx = target_x_clone.clone();
                fltk::app::add_timeout3(0.0, move |_| {
                    slide_tick(sl.clone(), tx.clone(), seg_y, btn_w, seg_h);
                });
            });
        }
    }

    pub fn set_callback<F>(&mut self, index: usize, mut user_cb: F)
    where
        F: FnMut(usize) + 'static,
    {
        let mut clones: Vec<button::Button> = self.buttons.clone();
        let target_x_clone = self.target_x.clone();
        let slider_clone = self.slider.clone();
        let dest_x = self.seg_x + self.btn_w * index as i32;
        let seg_y = self.seg_y;
        let btn_w = self.btn_w;
        let seg_h = self.seg_h;

        self.buttons[index].set_callback(move |_| {
            for (j, b) in clones.iter_mut().enumerate() {
                let lc = if j == index {
                    Color::White
                } else {
                    Color::from_rgb(160, 160, 160)
                };
                b.set_label_color(lc);
                b.redraw();
            }
            target_x_clone.store(dest_x, Ordering::SeqCst);
            let sl = slider_clone.clone();
            let tx = target_x_clone.clone();
            fltk::app::add_timeout3(0.0, move |_| {
                slide_tick(sl.clone(), tx.clone(), seg_y, btn_w, seg_h);
            });

            user_cb(index);
        });
    }
}

pub fn slide_tick(
    mut slider: frame::Frame,
    target_x: Arc<AtomicI32>,
    seg_y: i32,
    btn_w: i32,
    seg_h: i32,
) {
    let tx = target_x.load(Ordering::SeqCst);
    let cx = slider.x();
    let diff = tx - cx;

    if diff.abs() <= 1 {
        slider.resize(tx, seg_y, btn_w, seg_h);
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
    slider.resize(cx + step, seg_y, btn_w, seg_h);
    if let Some(mut p) = slider.parent() {
        p.redraw();
    }

    fltk::app::add_timeout3(frame_time, move |_| {
        slide_tick(slider.clone(), target_x.clone(), seg_y, btn_w, seg_h);
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

pub fn build_input(x: i32, y: i32, w: i32, h: i32) -> fltk::input::Input {
    let mut inp = fltk::input::Input::default().with_size(w, h).with_pos(x, y);
    inp.set_color(col(crate::ui::CLR_WIDGET));
    inp.set_text_color(Color::White);
    inp.set_text_font(Font::Helvetica);
    inp.set_text_size(13);
    inp.set_frame(FrameType::RFlatBox);
    inp.clear_visible_focus();
    inp
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

pub fn build_action_button(
    text: &str,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    color: Color,
    hover_color: Color,
) -> button::Button {
    let mut btn = button::Button::default().with_size(w, h).with_pos(x, y);
    btn.set_label(text);
    btn.set_label_font(Font::HelveticaBold);
    btn.set_label_size(13);
    btn.set_color(color);
    btn.set_selection_color(hover_color);
    btn.set_label_color(Color::White);
    btn.set_frame(FrameType::RFlatBox);
    btn.clear_visible_focus();
    btn
}
