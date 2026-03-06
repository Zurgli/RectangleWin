//! WinEvent hook for EVENT_SYSTEM_MOVESIZEEND: when the user finishes moving/resizing
//! a window we had snapped, restore it to the saved "non-snapped" rect.
//! (Restoring during drag is not possible: the OS overwrites our resize until the drag ends.)

use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Manager;
use std::sync::Mutex;
use std::thread;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::CoInitializeEx;
use windows::Win32::System::Com::COINIT_APARTMENTTHREADED;
use windows::Win32::UI::WindowsAndMessaging::{
    GetMessageW, TranslateMessage, DispatchMessageW,
    CreateWindowExW, DefWindowProcW, RegisterClassW,
    GetAncestor, IsWindow,
    GA_ROOT,
    WNDCLASSW, WS_EX_TOOLWINDOW, WS_OVERLAPPED,
};
use windows::core::w;

// Manual FFI for SetWinEventHook / UnhookWinEvent (not in our windows crate feature set).
const EVENT_SYSTEM_MOVESIZEEND: u32 = 11;
const WINEVENT_OUTOFCONTEXT: u32 = 0;

type HWINEVENTHOOK = isize;
type WINEVENTPROC = Option<
    unsafe extern "system" fn(
        h_win_event_hook: HWINEVENTHOOK,
        event: u32,
        hwnd: HWND,
        id_object: i32,
        id_child: i32,
        id_event_thread: u32,
        dwms_event_time: u32,
    ),
>;

#[link(name = "user32")]
extern "system" {
    fn SetWinEventHook(
        event_min: u32,
        event_max: u32,
        hmod_win_event_proc: *const std::ffi::c_void,
        pfn_win_event_proc: WINEVENTPROC,
        id_process: u32,
        id_thread: u32,
        dw_flags: u32,
    ) -> HWINEVENTHOOK;
    fn UnhookWinEvent(h_win_event_hook: HWINEVENTHOOK) -> std::ffi::c_int;
}

static MOVE_SIZE_APP: Mutex<Option<tauri::AppHandle>> = Mutex::new(None);
static RUNNING: AtomicBool = AtomicBool::new(false);

unsafe extern "system" fn win_event_proc(
    _h_win_event_hook: HWINEVENTHOOK,
    _event: u32,
    hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _id_event_thread: u32,
    _dwms_event_time: u32,
) {
    if hwnd.0.is_null() {
        return;
    }
    let root = GetAncestor(hwnd, GA_ROOT);
    if root.0.is_null() || !IsWindow(root).as_bool() {
        return;
    }
    let hwnd_raw = root.0 as usize;
    if let Ok(guard) = MOVE_SIZE_APP.lock() {
        if let Some(ref handle) = *guard {
            let h_for_call = handle.clone();
            let h_for_closure = handle.clone();
            let _ = h_for_call.run_on_main_thread(move || {
                on_move_size_end(h_for_closure, hwnd_raw);
            });
        }
    }
}

/// Called on the main thread: if this window was snapped by us and the user moved/resized it
/// (current rect no longer matches what we set), restore to the saved non-snapped rect.
fn on_move_size_end(handle: tauri::AppHandle, hwnd_raw: usize) {
    let hwnd = HWND(hwnd_raw as *mut _);
    if hwnd.0.is_null() || unsafe { !IsWindow(hwnd).as_bool() } {
        return;
    }
    let key = hwnd_raw as isize;
    let rect_to_apply = {
        let state = handle.state::<crate::AppState>();
        let manager = match state.manager.try_lock() {
            Ok(m) => m,
            Err(_) => return,
        };
        let restore_rect = match manager.get_restore_rect(key) {
            Some(r) => r,
            None => return,
        };
        let last_action_rect = match manager.get_last_action_rect(key) {
            Some(r) => r,
            None => return,
        };
        let current = match super::try_get_window_bounds(hwnd, false) {
            Some(r) => r,
            None => return,
        };
        const TOLERANCE: i32 = 5;
        if current.approximately_equals(&last_action_rect, TOLERANCE) {
            return; // user didn't move it
        }
        // Restore saved size but keep window at release position (title bar stays where user dropped it).
        crate::rect::Rect {
            left: current.left,
            top: current.top,
            right: current.left + restore_rect.width(),
            bottom: current.top + restore_rect.height(),
        }
    };
    let _ = super::set_window_bounds(hwnd, &rect_to_apply, false, false);
    {
        let state = handle.state::<crate::AppState>();
        let _ = state.manager.try_lock().map(|mut manager| manager.remove_last_action(key));
    }
}

unsafe extern "system" fn hook_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

/// Start the MoveSizeEnd hook. Call once from main thread (e.g. in setup).
/// When the user finishes moving/resizing a window we had snapped, we restore it to the saved rect.
pub fn start(app: tauri::AppHandle) {
    if RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }
    if let Ok(mut guard) = MOVE_SIZE_APP.lock() {
        *guard = Some(app);
    }

    thread::spawn(|| {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

            let hmod = windows::Win32::System::LibraryLoader::GetModuleHandleW(None)
                .expect("GetModuleHandleW");
            let hinstance = std::mem::transmute::<_, windows::Win32::Foundation::HINSTANCE>(hmod);

            let class_name = w!("RectangleWinMoveSizeHook");
            let mut wc = WNDCLASSW::default();
            wc.lpfnWndProc = Some(hook_wndproc);
            wc.hInstance = hinstance;
            wc.lpszClassName = class_name;
            let _ = RegisterClassW(&wc);

            let _hwnd = CreateWindowExW(
                WS_EX_TOOLWINDOW,
                class_name,
                w!(""),
                WS_OVERLAPPED,
                0,
                0,
                0,
                0,
                HWND::default(),
                windows::Win32::UI::WindowsAndMessaging::HMENU::default(),
                hinstance,
                None,
            )
            .expect("CreateWindowExW move/size hook window");

            let hook = SetWinEventHook(
                EVENT_SYSTEM_MOVESIZEEND,
                EVENT_SYSTEM_MOVESIZEEND,
                std::ptr::null(),
                Some(win_event_proc),
                0,
                0,
                WINEVENT_OUTOFCONTEXT,
            );
            if hook == 0 {
                RUNNING.store(false, Ordering::SeqCst);
                return;
            }

            let mut msg = std::mem::zeroed();
            while GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            let _ = UnhookWinEvent(hook);
        }
        RUNNING.store(false, Ordering::SeqCst);
    });
}
