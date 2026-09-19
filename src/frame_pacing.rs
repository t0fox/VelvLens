use std::time::Duration;

pub const DEFAULT_REFRESH_RATE_MILLIHERTZ: u32 = 60_000;
const MIN_REFRESH_RATE_MILLIHERTZ: u32 = 30_000;
const MAX_REFRESH_RATE_MILLIHERTZ: u32 = 360_000;

/// Returns one display interval for the requested monitor refresh rate.
///
/// The value is expressed in millihertz because Windows reports refresh rates
/// with sub-hertz precision on some displays. Unknown or invalid values use a
/// conservative 60 Hz fallback.
pub fn repaint_interval(refresh_rate_millihertz: Option<u32>) -> Duration {
    let refresh_rate_millihertz = refresh_rate_millihertz
        .filter(|rate| *rate > 0)
        .unwrap_or(DEFAULT_REFRESH_RATE_MILLIHERTZ)
        .clamp(MIN_REFRESH_RATE_MILLIHERTZ, MAX_REFRESH_RATE_MILLIHERTZ);
    Duration::from_nanos(1_000_000_000_000u64 / u64::from(refresh_rate_millihertz))
}

pub fn repaint_interval_for_frame(frame: &eframe::Frame) -> Duration {
    repaint_interval(monitor_refresh_rate_millihertz(frame))
}

#[cfg(windows)]
fn monitor_refresh_rate_millihertz(frame: &eframe::Frame) -> Option<u32> {
    use std::{ffi::c_void, mem::size_of};

    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows_sys::Win32::Graphics::Gdi::{
        EnumDisplaySettingsW, GetMonitorInfoW, MonitorFromWindow, DEVMODEW, ENUM_CURRENT_SETTINGS,
        MONITORINFO, MONITORINFOEXW, MONITOR_DEFAULTTONEAREST,
    };

    let window_handle = frame.window_handle().ok()?;
    let RawWindowHandle::Win32(window) = window_handle.as_raw() else {
        return None;
    };
    let hwnd = window.hwnd.get() as *mut c_void;

    // SAFETY: the handle comes from eframe's live native window. The Win32
    // structures are initialized with their documented sizes before use.
    unsafe {
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        if monitor.is_null() {
            return None;
        }

        let mut monitor_info: MONITORINFOEXW = std::mem::zeroed();
        monitor_info.monitorInfo.cbSize = size_of::<MONITORINFO>() as u32;
        if GetMonitorInfoW(monitor, &mut monitor_info.monitorInfo as *mut MONITORINFO) == 0 {
            return None;
        }

        let mut display_mode: DEVMODEW = std::mem::zeroed();
        display_mode.dmSize = size_of::<DEVMODEW>() as u16;
        if EnumDisplaySettingsW(
            monitor_info.szDevice.as_ptr(),
            ENUM_CURRENT_SETTINGS,
            &mut display_mode,
        ) == 0
        {
            return None;
        }

        let refresh_rate = display_mode.dmDisplayFrequency;
        (refresh_rate > 0).then_some(refresh_rate.saturating_mul(1_000))
    }
}

#[cfg(not(windows))]
fn monitor_refresh_rate_millihertz(_frame: &eframe::Frame) -> Option<u32> {
    None
}
