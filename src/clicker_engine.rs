use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering};
use std::time::{Duration, Instant};

pub const EXTRA_INFO: usize = 0xBADC0FFE;
const INPUT_MOUSE: u32 = 0;
const MOUSEEVENTF_LEFTDOWN: u32 = 0x0002;
const MOUSEEVENTF_LEFTUP: u32 = 0x0004;
const MOUSEEVENTF_RIGHTDOWN: u32 = 0x0008;
const MOUSEEVENTF_RIGHTUP: u32 = 0x0010;
const MOUSEEVENTF_MIDDLEDOWN: u32 = 0x0020;
const MOUSEEVENTF_MIDDLEUP: u32 = 0x0040;

#[repr(C)]
#[derive(Clone, Copy)]
struct MouseInput {
    dx: i32,
    dy: i32,
    mouse_data: u32,
    dw_flags: u32,
    time: u32,
    dw_extra_info: usize,
}
#[repr(C)]
struct Input {
    r#type: u32,
    mi: MouseInput,
}

#[link(name = "user32")]
unsafe extern "system" {
    fn SendInput(cInputs: u32, pInputs: *const Input, cbSize: i32) -> u32;
}

#[link(name = "ntdll")]
unsafe extern "system" {
    fn NtSetTimerResolution(desired: u32, set_resolution: u8, actual_resolution: *mut u32) -> i32;
}

struct TimerResolutionGuard;

impl TimerResolutionGuard {
    fn new() -> Self {
        let mut actual: u32 = 0;
        unsafe {
            NtSetTimerResolution(10_000, 1, &mut actual);
        }
        TimerResolutionGuard
    }
}

impl Drop for TimerResolutionGuard {
    fn drop(&mut self) {
        let mut actual: u32 = 0;
        unsafe {
            NtSetTimerResolution(10_000, 0, &mut actual);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClickButton {
    Left,
    Middle,
    Right,
}

impl ClickButton {
    fn flags(self) -> (u32, u32) {
        match self {
            ClickButton::Left => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
            ClickButton::Middle => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
            ClickButton::Right => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
        }
    }

    pub fn from_index(idx: i32) -> Self {
        match idx {
            1 => ClickButton::Middle,
            2 => ClickButton::Right,
            _ => ClickButton::Left,
        }
    }
}

fn send_click(button: ClickButton) {
    let (down, up) = button.flags();

    let inputs = [
        Input {
            r#type: INPUT_MOUSE,
            mi: MouseInput {
                dx: 0,
                dy: 0,
                mouse_data: 0,
                dw_flags: down,
                time: 0,
                dw_extra_info: EXTRA_INFO,
            },
        },
        Input {
            r#type: INPUT_MOUSE,
            mi: MouseInput {
                dx: 0,
                dy: 0,
                mouse_data: 0,
                dw_flags: up,
                time: 0,
                dw_extra_info: EXTRA_INFO,
            },
        },
    ];

    unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            std::mem::size_of::<Input>() as i32,
        );
    }
}

fn wait_until(
    mut deadline: Instant,
    interval_ms: &AtomicI32,
    is_active: &AtomicBool,
    generation: &AtomicU64,
    worker_generation: u64,
) -> bool {
    const TICK: Duration = Duration::from_millis(5);
    let mut interval = current_interval(interval_ms);

    loop {
        if !is_active.load(Ordering::Relaxed)
            || generation.load(Ordering::Acquire) != worker_generation
        {
            return false;
        }
        let now = Instant::now();
        let updated_interval = current_interval(interval_ms);
        if updated_interval != interval {
            interval = updated_interval;
            deadline = now + interval;
        }
        if now >= deadline {
            return true;
        }
        let remaining = deadline - now;
        std::thread::sleep(remaining.min(TICK));
    }
}

fn current_interval(interval_ms: &AtomicI32) -> Duration {
    Duration::from_millis(
        interval_ms
            .load(Ordering::Relaxed)
            .max(crate::speed::MIN_INTERVAL_MS) as u64,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FilterDecision {
    Allow,
    Stop,
    Pause,
}

pub fn start(
    interval_ms: Arc<AtomicI32>,
    button_index: Arc<AtomicI32>,
    is_active: Arc<AtomicBool>,
    filter_mode: Arc<AtomicI32>,
    processes: Arc<std::sync::Mutex<Vec<crate::settings_io::ProcessEntry>>>,
    clicker_hwnd: isize,
    generation: Arc<AtomicU64>,
) {
    let worker_generation = generation.fetch_add(1, Ordering::AcqRel) + 1;

    std::thread::spawn(move || {
        let _timer_guard = TimerResolutionGuard::new();
        let interval = current_interval(&interval_ms);
        let mut next_click = Instant::now() + interval;

        loop {
            if !wait_until(
                next_click,
                &interval_ms,
                &is_active,
                &generation,
                worker_generation,
            ) {
                break;
            }

            let decision = check_filter(&filter_mode, &processes, clicker_hwnd);
            if !is_active.load(Ordering::Relaxed)
                || generation.load(Ordering::Acquire) != worker_generation
            {
                break;
            }

            match decision {
                FilterDecision::Allow => {
                    send_click(ClickButton::from_index(
                        button_index.load(Ordering::Relaxed),
                    ));
                }
                FilterDecision::Stop => {
                    is_active.store(false, Ordering::Relaxed);
                    break;
                }
                FilterDecision::Pause => {}
            }

            let interval = current_interval(&interval_ms);
            next_click = Instant::now() + interval;
        }
    });
}

pub fn stop(generation: &AtomicU64) {
    generation.fetch_add(1, Ordering::AcqRel);
}

fn check_filter(
    filter_mode: &Arc<AtomicI32>,
    processes: &Arc<std::sync::Mutex<Vec<crate::settings_io::ProcessEntry>>>,
    clicker_hwnd: isize,
) -> FilterDecision {
    let fg_hwnd = crate::platform::get_foreground_window();
    if fg_hwnd.is_null() {
        return FilterDecision::Allow;
    }
    if fg_hwnd as isize == clicker_hwnd {
        return FilterDecision::Pause;
    }

    let mode = filter_mode.load(Ordering::Relaxed);
    let entries = match processes.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    if !entries.iter().any(|entry| entry.enabled) {
        return decide_filter(mode, &entries, "", "");
    }

    let title = crate::platform::get_window_title(fg_hwnd).to_lowercase();
    let proc_name = crate::platform::get_window_process_name(fg_hwnd).to_lowercase();
    decide_filter(mode, &entries, &title, &proc_name)
}

fn decide_filter(
    mode: i32,
    entries: &[crate::settings_io::ProcessEntry],
    title: &str,
    process_name: &str,
) -> FilterDecision {
    let matched_entry = entries
        .iter()
        .filter(|e| e.enabled)
        .find(|e| e.matches_normalized(title, process_name));

    match mode {
        0 => match matched_entry {
            Some(_) => FilterDecision::Allow,
            None => FilterDecision::Pause,
        },
        _ => match matched_entry {
            None => FilterDecision::Allow,
            Some(entry) => {
                if entry.action == 0 {
                    FilterDecision::Stop
                } else {
                    FilterDecision::Pause
                }
            }
        },
    }
}

pub fn start_from_atomics(
    interval_ms_atomic: Arc<AtomicI32>,
    button_index_atomic: Arc<AtomicI32>,
    is_active: Arc<AtomicBool>,
    filter_mode: Arc<AtomicI32>,
    processes: Arc<std::sync::Mutex<Vec<crate::settings_io::ProcessEntry>>>,
    clicker_hwnd: isize,
    generation: Arc<AtomicU64>,
) {
    start(
        interval_ms_atomic,
        button_index_atomic,
        is_active,
        filter_mode,
        processes,
        clicker_hwnd,
        generation,
    );
}


