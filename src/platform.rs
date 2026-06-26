use fltk::{prelude::*, window};

#[cfg(target_os = "windows")]
#[link(name = "dwmapi")]
extern "system" {
    /// Sets a DWM window attribute.
    /// See: https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/nf-dwmapi-dwmsetwindowattribute
    fn DwmSetWindowAttribute(
        hwnd: *mut std::ffi::c_void,
        dwAttribute: u32,
        pvAttribute: *const std::ffi::c_void,
        cbAttribute: u32,
    ) -> i32;
}

#[cfg(target_os = "windows")]
#[link(name = "user32")]
extern "system" {
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
}

/// Applies native Windows 11 rounded corners and the system accent border
/// using the DWM API. Falls back silently on older Windows versions.
pub fn apply_windows_style(wind: &mut window::Window) {
    #[cfg(target_os = "windows")]
    {
        let hwnd = wind.raw_handle();
        if !hwnd.is_null() {
            unsafe {
                // DWMWA_WINDOW_CORNER_PREFERENCE = 33
                // DWMWCP_ROUND = 2  (same rounded corners Windows uses for all its own windows)
                let corner_pref: u32 = 2;
                DwmSetWindowAttribute(
                    hwnd,
                    33,
                    &corner_pref as *const u32 as *const std::ffi::c_void,
                    std::mem::size_of::<u32>() as u32,
                );

                // DWMWA_BORDER_COLOR = 34
                // COLORREF format is 0x00BBGGRR; RGB(30,30,30) = 0x001E1E1E
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
    #[cfg(not(target_os = "windows"))]
    {
        let _ = wind;
    }
}

pub fn set_window_topmost(wind: &mut window::Window, topmost: bool) {
    #[cfg(target_os = "windows")]
    {
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
    #[cfg(not(target_os = "windows"))]
    {
        let _ = wind;
        let _ = topmost;
    }
}

pub fn show_in_taskbar(wind: &mut window::Window) {
    #[cfg(target_os = "windows")]
    {
        let hwnd = wind.raw_handle();
        if !hwnd.is_null() {
            unsafe {
                let style = GetWindowLongPtrW(hwnd, -20); // GWL_EXSTYLE = -20
                SetWindowLongPtrW(hwnd, -20, style | 0x00040000); // WS_EX_APPWINDOW = 0x00040000
                // Refresh window frame so Windows re-reads the new style
                SetWindowPos(
                    hwnd,
                    std::ptr::null_mut(),
                    0,
                    0,
                    0,
                    0,
                    0x0020 | 0x0002 | 0x0001 | 0x0004 | 0x0010 | 0x0200,
                );
                // Hide and show the window to force the Shell to update the taskbar
                ShowWindow(hwnd, 0); // SW_HIDE = 0
                ShowWindow(hwnd, 5); // SW_SHOW = 5
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = wind;
    }
}
