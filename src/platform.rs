use fltk::{prelude::*, window};
use std::sync::atomic::{AtomicPtr, Ordering as AOrdering};
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;

#[link(name = "dwmapi")]
unsafe extern "system" {

    fn DwmSetWindowAttribute(
        hwnd: *mut std::ffi::c_void,
        dwAttribute: u32,
        pvAttribute: *const std::ffi::c_void,
        cbAttribute: u32,
    ) -> i32;
}

#[repr(C)]
struct KbdllHookStruct {
    vk_code: u32,
    scan_code: u32,
    flags: u32,
    time: u32,
    dw_extra_info: usize,
}

std::thread_local! {
    static HOOK_SENDER: std::cell::RefCell<Option<mpsc::SyncSender<u32>>> =
        std::cell::RefCell::new(None);
}

static HOOK_HANDLE: AtomicPtr<std::ffi::c_void> = AtomicPtr::new(std::ptr::null_mut());

#[link(name = "user32")]
unsafe extern "system" {
    fn SetWindowsHookExW(
        id_hook: i32,
        lpfn: unsafe extern "system" fn(i32, usize, isize) -> isize,
        h_mod: *mut std::ffi::c_void,
        dw_thread_id: u32,
    ) -> *mut std::ffi::c_void;

    fn UnhookWindowsHookEx(hhk: *mut std::ffi::c_void) -> i32;

    fn CallNextHookEx(
        hhk: *mut std::ffi::c_void,
        n_code: i32,
        w_param: usize,
        l_param: isize,
    ) -> isize;

    fn GetMessageW(
        lp_msg: *mut [u32; 7],
        hwnd: *mut std::ffi::c_void,
        w_msg_filter_min: u32,
        w_msg_filter_max: u32,
    ) -> i32;

    fn PostThreadMessageW(id_thread: u32, msg: u32, w_param: usize, l_param: isize) -> i32;

    fn GetCurrentThreadId() -> u32;

    fn GetDC(hWnd: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn ReleaseDC(hWnd: *mut std::ffi::c_void, hDC: *mut std::ffi::c_void) -> i32;
}

#[link(name = "gdi32")]
unsafe extern "system" {
    fn GetDeviceCaps(hDC: *mut std::ffi::c_void, nIndex: i32) -> i32;
}

pub fn get_monitor_refresh_rate() -> u32 {
    unsafe {
        let dc = GetDC(std::ptr::null_mut());
        if dc.is_null() {
            return 60;
        }

        let rate = GetDeviceCaps(dc, 116);
        ReleaseDC(std::ptr::null_mut(), dc);
        if rate > 0 {
            rate as u32
        } else {
            60
        }
    }
}

unsafe extern "system" fn ll_keyboard_proc(n_code: i32, w_param: usize, l_param: isize) -> isize {
    unsafe {
        if n_code >= 0 && (w_param == 0x0100 || w_param == 0x0104) {
            let info = &*(l_param as *const KbdllHookStruct);

            if info.dw_extra_info != crate::clicker_engine::EXTRA_INFO {
                HOOK_SENDER.with(|cell| {
                    if let Some(tx) = cell.borrow().as_ref() {
                        let _ = tx.try_send(info.vk_code);
                    }
                });
            }
        }
        CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param)
    }
}

pub struct GlobalHookGuard {
    pump_thread_id: u32,
}

impl Drop for GlobalHookGuard {
    fn drop(&mut self) {
        unsafe {
            let hhk = HOOK_HANDLE.load(AOrdering::SeqCst);
            if !hhk.is_null() {
                UnhookWindowsHookEx(hhk);
                HOOK_HANDLE.store(std::ptr::null_mut(), AOrdering::SeqCst);
            }

            PostThreadMessageW(self.pump_thread_id, 0x0012, 0, 0);
        }
    }
}

pub fn install_global_hook() -> (GlobalHookGuard, Receiver<u32>) {
    let (tx, rx) = mpsc::sync_channel::<u32>(32);
    let pump_tid = Arc::new(std::sync::atomic::AtomicU32::new(0));
    let pump_tid_clone = pump_tid.clone();

    std::thread::spawn(move || {
        HOOK_SENDER.with(|cell| {
            *cell.borrow_mut() = Some(tx);
        });

        let tid = unsafe { GetCurrentThreadId() };
        pump_tid_clone.store(tid, AOrdering::SeqCst);

        let hhk = unsafe { SetWindowsHookExW(13, ll_keyboard_proc, std::ptr::null_mut(), 0) };
        HOOK_HANDLE.store(hhk, AOrdering::SeqCst);

        let mut msg = [0u32; 7];
        loop {
            let ret = unsafe { GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) };
            if ret <= 0 {
                break;
            }
        }
    });

    while pump_tid.load(AOrdering::SeqCst) == 0 {
        std::hint::spin_loop();
    }
    let id = pump_tid.load(AOrdering::SeqCst);

    (GlobalHookGuard { pump_thread_id: id }, rx)
}

#[link(name = "user32")]
unsafe extern "system" {
    fn SetWindowPos(
        hWnd: *mut std::ffi::c_void,
        hWndInsertAfter: *mut std::ffi::c_void,
        X: i32,
        Y: i32,
        cx: i32,
        cy: i32,
        uFlags: u32,
    ) -> i32;

    fn ShowWindow(hWnd: *mut std::ffi::c_void, nCmdShow: i32) -> i32;

    #[cfg(target_pointer_width = "64")]
    fn SetWindowLongPtrW(hWnd: *mut std::ffi::c_void, nIndex: i32, dwNewLong: isize) -> isize;
    #[cfg(target_pointer_width = "64")]
    fn GetWindowLongPtrW(hWnd: *mut std::ffi::c_void, nIndex: i32) -> isize;

    #[cfg(target_pointer_width = "32")]
    #[link_name = "SetWindowLongW"]
    fn SetWindowLongPtrW(hWnd: *mut std::ffi::c_void, nIndex: i32, dwNewLong: isize) -> isize;
    #[cfg(target_pointer_width = "32")]
    #[link_name = "GetWindowLongW"]
    fn GetWindowLongPtrW(hWnd: *mut std::ffi::c_void, nIndex: i32) -> isize;

    fn GetForegroundWindow() -> *mut std::ffi::c_void;
}

pub fn apply_windows_style(wind: &mut window::Window) {
    let hwnd = wind.raw_handle();
    if !hwnd.is_null() {
        unsafe {
            let corner_pref: u32 = 2;
            DwmSetWindowAttribute(
                hwnd,
                33,
                &corner_pref as *const u32 as *const std::ffi::c_void,
                std::mem::size_of::<u32>() as u32,
            );

            let border_color: u32 = 0x001E_1E1E;
            DwmSetWindowAttribute(
                hwnd,
                34,
                &border_color as *const u32 as *const std::ffi::c_void,
                std::mem::size_of::<u32>() as u32,
            );
        }
    }
}

pub fn set_window_topmost(wind: &mut window::Window, topmost: bool) {
    let hwnd = wind.raw_handle();
    if !hwnd.is_null() {
        let hwnd_insert_after = if topmost {
            -1isize as *mut std::ffi::c_void
        } else {
            -2isize as *mut std::ffi::c_void
        };
        unsafe {
            SetWindowPos(hwnd, hwnd_insert_after, 0, 0, 0, 0, 0x0001 | 0x0002);
        }
    }
}

pub fn show_in_taskbar(wind: &mut window::Window) {
    let hwnd = wind.raw_handle();
    if !hwnd.is_null() {
        unsafe {
            let style = GetWindowLongPtrW(hwnd, -20);
            SetWindowLongPtrW(hwnd, -20, style | 0x00040000);

            SetWindowPos(
                hwnd,
                std::ptr::null_mut(),
                0,
                0,
                0,
                0,
                0x0020 | 0x0002 | 0x0001 | 0x0004 | 0x0010 | 0x0200,
            );

            ShowWindow(hwnd, 0);
            ShowWindow(hwnd, 5);
        }
    }
}

pub fn hide_window(wind: &mut window::Window) {
    let hwnd = wind.raw_handle();
    if !hwnd.is_null() {
        unsafe {
            ShowWindow(hwnd, 0);
        }
    }
}

pub fn minimize_window(wind: &mut window::Window) {
    let hwnd = wind.raw_handle();
    if !hwnd.is_null() {
        unsafe {
            ShowWindow(hwnd, 6);
        }
    }
}

pub fn show_window(wind: &mut window::Window) {
    let hwnd = wind.raw_handle();
    if !hwnd.is_null() {
        unsafe {
            ShowWindow(hwnd, 5);
            ShowWindow(hwnd, 9);
        }
    }
}

pub fn get_foreground_window() -> *mut std::ffi::c_void {
    unsafe { GetForegroundWindow() }
}
