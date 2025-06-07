use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows::{
    Win32::Foundation::HWND,
    Win32::Graphics::Dwm::{
        DWMNCRP_DISABLED, DWMWA_NCRENDERING_POLICY, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND,
        DwmSetWindowAttribute,
    },
};

#[cfg(target_os = "windows")]
pub fn set_rounded_corners(hwnd: HWND) {
    unsafe {
        let preference = DWMWCP_ROUND;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &preference as *const _ as *const _,
            std::mem::size_of::<u32>() as u32,
        );
    }
}

#[cfg(target_os = "windows")]
pub fn disable_dwm_shadow(hwnd: HWND) {
    unsafe {
        let value: i32 = DWMNCRP_DISABLED.0;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_NCRENDERING_POLICY,
            &value as *const _ as _,
            std::mem::size_of::<i32>() as u32,
        );
    }
}

#[cfg(target_os = "windows")]
pub fn apply_window_styling(window_handle: &dyn HasWindowHandle) {
    if let Ok(raw_handle) = window_handle.window_handle() {
        if let RawWindowHandle::Win32(win32_handle) = raw_handle.as_raw() {
            let hwnd = HWND(win32_handle.hwnd.get());
            set_rounded_corners(hwnd);
            disable_dwm_shadow(hwnd);
        }
    }
}
