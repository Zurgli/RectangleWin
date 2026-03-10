//! Windows implementation of Win32 wrappers.

use crate::rect::Rect;
use windows::Win32::Foundation::{CloseHandle, BOOL, HWND, LPARAM, POINT, RECT as WinRect};
use windows::Win32::Graphics::Dwm::{
    DwmGetWindowAttribute, DwmSetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS,
    DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_DONOTROUND, DWM_WINDOW_CORNER_PREFERENCE,
};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, MonitorFromPoint, MonitorFromWindow, HMONITOR,
    MONITORINFOEXW, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_NATIVE, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetAncestor, GetCursorPos, GetForegroundWindow, GetWindowLongW, GetWindowRect,
    GetWindowThreadProcessId, IsWindow, IsWindowVisible, SetCursorPos, SetForegroundWindow,
    SetWindowPos, GA_ROOT, GWL_EXSTYLE, GWL_STYLE, SET_WINDOW_POS_FLAGS, WS_CAPTION,
    WS_EX_TOOLWINDOW, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_POPUP, WS_THICKFRAME, WS_VISIBLE,
};

const SWP_NOZORDER_U: u32 = 0x0004;
const SWP_NOACTIVATE_U: u32 = 0x0010;

fn win_rect_to_rect(r: &WinRect) -> Rect {
    Rect {
        left: r.left,
        top: r.top,
        right: r.right,
        bottom: r.bottom,
    }
}

fn rect_to_win_rect(r: &Rect) -> WinRect {
    WinRect {
        left: r.left,
        top: r.top,
        right: r.right,
        bottom: r.bottom,
    }
}

fn has_style(style: u32, flag: u32) -> bool {
    (style & flag) == flag
}

fn is_non_processable_popup(style: u32) -> bool {
    let is_popup = has_style(style, WS_POPUP.0);
    let has_thick_frame = has_style(style, WS_THICKFRAME.0);
    let has_caption = has_style(style, WS_CAPTION.0);
    let has_minimize_or_maximize =
        has_style(style, WS_MINIMIZEBOX.0) || has_style(style, WS_MAXIMIZEBOX.0);

    is_popup && !(has_thick_frame && (has_caption || has_minimize_or_maximize))
}

fn is_snap_target_window(hwnd: HWND) -> bool {
    unsafe {
        if hwnd.0.is_null() || !IsWindow(hwnd).as_bool() || !IsWindowVisible(hwnd).as_bool() {
            return false;
        }

        let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;

        if !has_style(style, WS_VISIBLE.0) {
            return false;
        }
        if has_style(ex_style, WS_EX_TOOLWINDOW.0) {
            return false;
        }
        if is_non_processable_popup(style) {
            return false;
        }

        true
    }
}

/// Resolve a caller-provided HWND to a snap target window, filtering out shell popups
/// and utility surfaces like the Start menu or tool windows.
pub fn resolve_snap_target_window(hwnd: HWND) -> Option<HWND> {
    unsafe {
        if hwnd.0.is_null() || !IsWindow(hwnd).as_bool() {
            return None;
        }

        let root = GetAncestor(hwnd, GA_ROOT);
        let candidate = if root.0.is_null() || !IsWindow(root).as_bool() {
            hwnd
        } else {
            root
        };

        is_snap_target_window(candidate).then_some(candidate)
    }
}

/// Get foreground window handle (root/top-level), or None if invalid.
/// Uses GetAncestor(GA_ROOT) so we always move the top-level window, not a child
/// (e.g. when the Tauri/WebView window is focused, the foreground may be the WebView child).
pub fn get_foreground_window() -> Option<HWND> {
    unsafe { resolve_snap_target_window(GetForegroundWindow()) }
}

/// Get window bounds (prefer DWM extended frame bounds).
pub fn try_get_window_bounds(hwnd: HWND, use_window_rect: bool) -> Option<Rect> {
    unsafe {
        if hwnd.0.is_null() || !IsWindow(hwnd).as_bool() {
            return None;
        }
        if use_window_rect {
            let mut r = WinRect::default();
            if GetWindowRect(hwnd, &mut r).is_ok() {
                return Some(win_rect_to_rect(&r));
            }
            return None;
        }
        let mut frame = WinRect::default();
        if DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            &mut frame as *mut _ as *mut _,
            std::mem::size_of::<WinRect>() as u32,
        )
        .is_ok()
        {
            return Some(win_rect_to_rect(&frame));
        }
        let mut r = WinRect::default();
        if GetWindowRect(hwnd, &mut r).is_ok() {
            return Some(win_rect_to_rect(&r));
        }
        None
    }
}

pub fn get_window_corner_preference(hwnd: HWND) -> Option<DWM_WINDOW_CORNER_PREFERENCE> {
    unsafe {
        if hwnd.0.is_null() || !IsWindow(hwnd).as_bool() {
            return None;
        }

        let mut preference = DWM_WINDOW_CORNER_PREFERENCE(0);
        DwmGetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &mut preference as *mut _ as *mut _,
            std::mem::size_of::<DWM_WINDOW_CORNER_PREFERENCE>() as u32,
        )
        .ok()?;

        Some(preference)
    }
}

pub fn set_window_corner_preference(hwnd: HWND, preference: DWM_WINDOW_CORNER_PREFERENCE) -> bool {
    unsafe {
        if hwnd.0.is_null() || !IsWindow(hwnd).as_bool() {
            return false;
        }

        DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &preference as *const _ as *const _,
            std::mem::size_of::<DWM_WINDOW_CORNER_PREFERENCE>() as u32,
        )
        .is_ok()
    }
}

pub fn remove_rounded_corners(hwnd: HWND) -> bool {
    set_window_corner_preference(hwnd, DWMWCP_DONOTROUND)
}

/// Set window position and size. When rect_is_visible_bounds, convert visible rect to window rect using DWM frame offset.
pub fn set_window_bounds(
    hwnd: HWND,
    rect: &Rect,
    activate: bool,
    rect_is_visible_bounds: bool,
) -> bool {
    unsafe {
        if hwnd.0.is_null() || !IsWindow(hwnd).as_bool() {
            return false;
        }
        let mut to_set = rect_to_win_rect(rect);
        if rect_is_visible_bounds {
            let mut window_rect = WinRect::default();
            let mut frame = WinRect::default();
            let dwm_ok = GetWindowRect(hwnd, &mut window_rect).is_ok()
                && DwmGetWindowAttribute(
                    hwnd,
                    DWMWA_EXTENDED_FRAME_BOUNDS,
                    &mut frame as *mut _ as *mut _,
                    std::mem::size_of::<WinRect>() as u32,
                )
                .is_ok();
            if dwm_ok {
                let border_left = frame.left - window_rect.left;
                let border_top = frame.top - window_rect.top;
                let border_right = window_rect.right - frame.right;
                let border_bottom = window_rect.bottom - frame.bottom;
                to_set = WinRect {
                    left: rect.left - border_left,
                    top: rect.top - border_top,
                    right: rect.right + border_right,
                    bottom: rect.bottom + border_bottom,
                };
            } else {
                // DWM failed (e.g. Tauri/WebView or custom frame): use default frame so we still move the window
                const DEFAULT_FRAME_LEFT: i32 = 8;
                const DEFAULT_FRAME_TOP: i32 = 31;
                const DEFAULT_FRAME_RIGHT: i32 = 8;
                const DEFAULT_FRAME_BOTTOM: i32 = 8;
                to_set = WinRect {
                    left: rect.left - DEFAULT_FRAME_LEFT,
                    top: rect.top - DEFAULT_FRAME_TOP,
                    right: rect.right + DEFAULT_FRAME_RIGHT,
                    bottom: rect.bottom + DEFAULT_FRAME_BOTTOM,
                };
            }
        }
        let flags = SWP_NOZORDER_U | if activate { 0 } else { SWP_NOACTIVATE_U };
        SetWindowPos(
            hwnd,
            windows::Win32::Foundation::HWND(std::ptr::null_mut()),
            to_set.left,
            to_set.top,
            to_set.right - to_set.left,
            to_set.bottom - to_set.top,
            SET_WINDOW_POS_FLAGS(flags),
        )
        .is_ok()
    }
}

pub fn set_foreground_window(hwnd: HWND) -> bool {
    unsafe { SetForegroundWindow(hwnd).as_bool() }
}

// --- Monitor ---

pub fn get_monitor_from_window(hwnd: HWND) -> HMONITOR {
    unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) }
}

pub fn get_monitor_from_point(x: i32, y: i32) -> HMONITOR {
    unsafe {
        let pt = POINT { x, y };
        MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST)
    }
}

pub fn try_get_monitor_info(hmonitor: HMONITOR) -> Option<(Rect, Rect)> {
    unsafe {
        let mut mi: MONITORINFOEXW = std::mem::zeroed();
        mi.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
        if GetMonitorInfoW(hmonitor, &mut mi.monitorInfo).as_bool() {
            let work = win_rect_to_rect(&mi.monitorInfo.rcWork);
            let mon = win_rect_to_rect(&mi.monitorInfo.rcMonitor);
            return Some((mon, work));
        }
        None
    }
}

pub struct MonitorInfo {
    pub hmonitor: HMONITOR,
    pub monitor_rect: Rect,
    pub work_area: Rect,
}

pub fn enum_monitors() -> Vec<MonitorInfo> {
    unsafe {
        let mut list: Vec<MonitorInfo> = Vec::new();
        let lp = &mut list as *mut Vec<MonitorInfo>;
        unsafe extern "system" fn enum_proc(
            hmonitor: HMONITOR,
            _hdc: windows::Win32::Graphics::Gdi::HDC,
            _lprc: *mut windows::Win32::Foundation::RECT,
            dw_data: LPARAM,
        ) -> BOOL {
            let list = &mut *(dw_data.0 as *mut Vec<MonitorInfo>);
            let mut mi: MONITORINFOEXW = std::mem::zeroed();
            mi.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
            if GetMonitorInfoW(hmonitor, &mut mi.monitorInfo).as_bool() {
                list.push(MonitorInfo {
                    hmonitor,
                    monitor_rect: win_rect_to_rect(&mi.monitorInfo.rcMonitor),
                    work_area: win_rect_to_rect(&mi.monitorInfo.rcWork),
                });
            }
            BOOL::from(true)
        }
        let _ = EnumDisplayMonitors(None, None, Some(enum_proc), LPARAM(lp as isize));
        list.sort_by(|a, b| {
            a.monitor_rect
                .left
                .cmp(&b.monitor_rect.left)
                .then(a.monitor_rect.top.cmp(&b.monitor_rect.top))
        });
        list
    }
}

// --- Cursor ---

pub fn get_cursor_pos() -> Option<(i32, i32)> {
    unsafe {
        let mut pt = POINT::default();
        if GetCursorPos(&mut pt).is_ok() {
            return Some((pt.x, pt.y));
        }
        None
    }
}

pub fn set_cursor_pos(x: i32, y: i32) -> bool {
    unsafe { SetCursorPos(x, y).is_ok() }
}

// --- Process ---

/// Get process image file name (exe name) for the window. Returns None on failure.
pub fn get_process_image_name(hwnd: HWND) -> Option<String> {
    unsafe {
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(std::ptr::addr_of_mut!(pid)));
        if pid == 0 {
            return None;
        }
        let h_process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 261];
        let mut size = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(
            h_process,
            PROCESS_NAME_NATIVE,
            windows::core::PWSTR(buf.as_mut_ptr()),
            &mut size,
        )
        .is_ok();
        let _ = CloseHandle(h_process);
        if !ok {
            return None;
        }
        let path = String::from_utf16_lossy(&buf[..size as usize]);
        let path = path.trim_matches('\0');
        if path.is_empty() {
            return None;
        }
        std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .map(String::from)
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::{is_non_processable_popup, is_snap_target_window, resolve_snap_target_window};
    use windows::core::w;
    use windows::Win32::Foundation::{HINSTANCE, HWND};
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassW, HMENU, WNDCLASSW,
        WS_CAPTION, WS_CLIPCHILDREN, WS_CLIPSIBLINGS, WS_EX_TOOLWINDOW, WS_MAXIMIZEBOX,
        WS_MINIMIZEBOX, WS_OVERLAPPED, WS_POPUP, WS_THICKFRAME, WS_VISIBLE,
    };

    #[test]
    fn popup_menu_styles_are_rejected() {
        let style = (WS_POPUP | WS_CLIPCHILDREN | WS_CLIPSIBLINGS | WS_VISIBLE).0;
        assert!(is_non_processable_popup(style));
    }

    #[test]
    fn popup_app_styles_are_allowed() {
        let style =
            (WS_POPUP | WS_THICKFRAME | WS_CAPTION | WS_MINIMIZEBOX | WS_MAXIMIZEBOX | WS_VISIBLE)
                .0;
        assert!(!is_non_processable_popup(style));
    }

    #[test]
    fn tool_windows_are_not_snap_targets() {
        unsafe {
            let class_name = w!("RectangleWinImplTests");
            let mut wc = WNDCLASSW::default();
            wc.lpfnWndProc = Some(wndproc);
            wc.lpszClassName = class_name;
            let _ = RegisterClassW(&wc);

            let hwnd = CreateWindowExW(
                WS_EX_TOOLWINDOW,
                class_name,
                w!("Tool"),
                WS_OVERLAPPED | WS_VISIBLE,
                0,
                0,
                100,
                100,
                HWND::default(),
                HMENU::default(),
                HINSTANCE::default(),
                None,
            )
            .expect("create tool window");

            assert!(!is_snap_target_window(hwnd));
            assert_eq!(resolve_snap_target_window(hwnd), None);

            let _ = DestroyWindow(hwnd);
        }
    }

    unsafe extern "system" fn wndproc(
        hwnd: HWND,
        msg: u32,
        wparam: windows::Win32::Foundation::WPARAM,
        lparam: windows::Win32::Foundation::LPARAM,
    ) -> windows::Win32::Foundation::LRESULT {
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}
