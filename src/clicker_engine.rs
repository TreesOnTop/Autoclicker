use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub const EXTRA_INFO: usize = 0xBADC0FFE;

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
            ClickButton::Left => (0x0002, 0x0004),

            ClickButton::Middle => (0x0020, 0x0040),

            ClickButton::Right => (0x0008, 0x0010),
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
            r#type: 0,
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
            r#type: 0,
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

fn wait_until(deadline: Instant, is_active: &AtomicBool) {
    const TICK: Duration = Duration::from_millis(5);

    loop {
        let now = Instant::now();
        if now >= deadline || !is_active.load(Ordering::Relaxed) {
            break;
        }
        let remaining = deadline - now;
        std::thread::sleep(remaining.min(TICK));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FilterDecision {
    Allow,
    Stop,
    Pause,
}

pub fn start(
    interval_ms: u64,
    button: ClickButton,
    is_active: Arc<AtomicBool>,
    filter_mode: Arc<AtomicI32>,
    processes: Arc<std::sync::Mutex<Vec<crate::settings_io::ProcessEntry>>>,
    clicker_hwnd: isize,
) {
    let interval = Duration::from_millis(interval_ms.max(1));

    std::thread::spawn(move || {
        let _timer_guard = TimerResolutionGuard::new();

        let mut next_click = Instant::now() + interval;

        while is_active.load(Ordering::Relaxed) {
            wait_until(next_click, &is_active);

            if !is_active.load(Ordering::Relaxed) {
                break;
            }

            let decision = check_filter(&filter_mode, &processes, clicker_hwnd);

            match decision {
                FilterDecision::Allow => {
                    send_click(button);
                }
                FilterDecision::Stop => {
                    is_active.store(false, Ordering::Relaxed);
                    break;
                }
                FilterDecision::Pause => {
                }
            }

            next_click += interval;
        }
    });
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

    let title = crate::platform::get_window_title(fg_hwnd).to_lowercase();
    let proc_name = crate::platform::get_window_process_name(fg_hwnd).to_lowercase();

    let mode = filter_mode.load(Ordering::SeqCst);

    let entries = match processes.lock() {
        Ok(guard) => guard.clone(),
        Err(_) => return FilterDecision::Allow,
    };

    let matched_entry = entries
        .iter()
        .filter(|e| e.enabled)
        .find(|e| {
            let name_lower = e.name.to_lowercase();
            !name_lower.is_empty()
                && (title.contains(&name_lower) || proc_name.contains(&name_lower))
        });

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
) {
    let interval_ms = interval_ms_atomic.load(Ordering::SeqCst).max(1) as u64;
    let button = ClickButton::from_index(button_index_atomic.load(Ordering::SeqCst));
    start(
        interval_ms,
        button,
        is_active,
        filter_mode,
        processes,
        clicker_hwnd,
    );
}
