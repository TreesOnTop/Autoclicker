use fltk::{button, frame, group, prelude::*, enums::*};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use crate::ui::{col, CLR_BADGE_INACTIVE, CLR_GREEN};



















pub struct SegmentedToggle {
    
    pub group: group::Group,
    
    pub buttons: Vec<button::Button>,
    
    pub active_idx: Arc<AtomicI32>,
}

impl SegmentedToggle {
    
    pub fn active_index(&self) -> usize {
        self.active_idx.load(Ordering::SeqCst) as usize
    }
}






pub fn build_segmented_toggle(
    x: i32,
    y: i32,
    total_w: i32,
    h: i32,
    labels: &[&str],
    initial_idx: usize,
) -> SegmentedToggle {
    assert!(!labels.is_empty(), "SegmentedToggle needs at least one label");
    let n = labels.len() as i32;
    let btn_w = total_w / n;
    let container_clr = col(CLR_BADGE_INACTIVE);
    let active_clr = col(CLR_GREEN);

    
    let mut group = group::Group::default()
        .with_size(total_w, h)
        .with_pos(x, y);
    group.set_frame(FrameType::RFlatBox);
    group.set_color(container_clr);

    
    let start_x = x + btn_w * initial_idx as i32;
    let mut slider = frame::Frame::default()
        .with_size(btn_w, h)
        .with_pos(start_x, y);
    slider.set_frame(FrameType::RFlatBox);
    slider.set_color(active_clr);

    
    let mut buttons: Vec<button::Button> = labels
        .iter()
        .enumerate()
        .map(|(i, &label)| {
            let mut b = button::Button::default()
                .with_size(btn_w, h)
                .with_pos(x + btn_w * i as i32, y);
            b.set_label(label);
            let lc = if i == initial_idx {
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

    group.end();

    
    let active_idx = Arc::new(AtomicI32::new(initial_idx as i32));
    
    let target_x = Arc::new(AtomicI32::new(start_x));

    
    for i in 0..buttons.len() {
        let mut clones = buttons.clone();
        let active_idx_clone = active_idx.clone();
        let target_x_clone = target_x.clone();
        let slider_clone = slider.clone();
        let dest_x = x + btn_w * i as i32;
        let seg_y = y;
        let seg_h = h;

        buttons[i].set_callback(move |_| {
            
            for (j, b) in clones.iter_mut().enumerate() {
                b.set_label_color(if j == i {
                    Color::White
                } else {
                    Color::from_rgb(160, 160, 160)
                });
                b.redraw();
            }
            active_idx_clone.store(i as i32, Ordering::SeqCst);

            
            target_x_clone.store(dest_x, Ordering::SeqCst);
            let sl = slider_clone.clone();
            let tx = target_x_clone.clone();
            fltk::app::add_timeout3(0.0, move |_| {
                seg_slide_tick(sl.clone(), tx.clone(), seg_y, btn_w, seg_h);
            });
        });
    }

    SegmentedToggle {
        group,
        buttons,
        active_idx,
    }
}







pub fn seg_slide_tick(
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
        seg_slide_tick(slider.clone(), target_x.clone(), seg_y, btn_w, seg_h);
    });
}
