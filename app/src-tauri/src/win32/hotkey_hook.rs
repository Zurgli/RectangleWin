//! Low-level keyboard hook (WH_KEYBOARD_LL) so we can capture Win+key and override
//! any other shortcuts. Runs on a dedicated thread with a hidden window and message loop.

use crate::config::HotkeyBinding;
use crate::shortcut;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
use tauri::Emitter;
use windows::core::w;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Com::CoInitializeEx;
use windows::Win32::System::Com::COINIT_APARTMENTTHREADED;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
    VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
    PostMessageW, PostQuitMessage, RegisterClassW, SetWindowsHookExW, TranslateMessage,
    UnhookWindowsHookEx, HC_ACTION, KBDLLHOOKSTRUCT, LLKHF_UP, WH_KEYBOARD_LL, WM_CLOSE,
    WM_DESTROY, WM_KEYDOWN, WM_SYSKEYDOWN, WNDCLASSW, WS_EX_TOOLWINDOW, WS_OVERLAPPED,
};

const MOD_MASK: u32 = 0x000F; // MOD_ALT | MOD_CONTROL | MOD_SHIFT | MOD_WIN
const MOD_ALT_BIT: u32 = 0x01;
const MOD_CONTROL_BIT: u32 = 0x02;
const MOD_SHIFT_BIT: u32 = 0x04;
const MOD_WIN_BIT: u32 = 0x08;
const VK_LWIN: u32 = 0x5B;
const VK_RWIN: u32 = 0x5C;
const VK_CONTROL: u32 = 0x11;
const VK_MENU: u32 = 0x12; // generic Alt
const VK_SHIFT: u32 = 0x10;
const VK_LMENU: u32 = 0xA4; // left Alt (hook can receive these instead of VK_MENU)
const VK_RMENU: u32 = 0xA5;
const VK_LCONTROL: u32 = 0xA2;
const VK_RCONTROL: u32 = 0xA3;
const VK_LSHIFT: u32 = 0xA0;
const VK_RSHIFT: u32 = 0xA1;
const VK_F24_CODE: u16 = 0x87;
const PROBE_EXTRA_INFO: usize = 0x5257_484b; // "RWHK"

static HOOK_STATE: Mutex<Option<HookState>> = Mutex::new(None);
/// usize = HWND.0 for passing across threads (HWND is !Send).
static HOOK_TRIGGER: Mutex<Option<Box<dyn Fn(String, Option<usize>) + Send>>> = Mutex::new(None);
/// When set (e.g. RECTANGLEWIN_DEBUG_KEYS=1), every intercepted key down is emitted to the frontend.
static DEBUG_APP: Mutex<Option<tauri::AppHandle>> = Mutex::new(None);
static RUNNING: AtomicBool = AtomicBool::new(false);
static HOOK_WINDOW: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);
static PROBE_ACKED: AtomicBool = AtomicBool::new(false);
const REPEAT_SUPPRESSION_WINDOW: Duration = Duration::from_millis(750);
const STOP_WAIT_SLICE: Duration = Duration::from_millis(25);
const PROBE_WAIT_SLICE: Duration = Duration::from_millis(10);

#[derive(serde::Serialize)]
struct KeyInterceptedPayload {
    vk: u32,
    modifiers: u32,
    label: String,
    is_hotkey: bool,
}

struct HookState {
    hotkeys: Vec<(u32, u32, String)>, // (modifiers & MOD_MASK, vk, action_name)
    last_triggered: Option<TriggeredHotkey>,
    current_modifiers: u32,
    last_callback_at: Instant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TriggeredHotkey {
    modifiers: u32,
    vk: u32,
    triggered_at: Instant,
}

fn should_consume_hotkey(matched_action: bool, hwnd_raw: Option<usize>) -> bool {
    matched_action && hwnd_raw.is_some()
}

fn modifier_bit_for_vk(vk: u32) -> Option<u32> {
    match vk {
        VK_LWIN | VK_RWIN => Some(MOD_WIN_BIT),
        VK_CONTROL | VK_LCONTROL | VK_RCONTROL => Some(MOD_CONTROL_BIT),
        VK_MENU | VK_LMENU | VK_RMENU => Some(MOD_ALT_BIT),
        VK_SHIFT | VK_LSHIFT | VK_RSHIFT => Some(MOD_SHIFT_BIT),
        _ => None,
    }
}

fn is_modifier_key(vk: u32) -> bool {
    modifier_bit_for_vk(vk).is_some()
}

fn update_modifier_state(state: &mut HookState, vk: u32, is_up: bool) {
    let Some(bit) = modifier_bit_for_vk(vk) else {
        return;
    };

    if is_up {
        state.current_modifiers &= !bit;
    } else {
        state.current_modifiers |= bit;
    }
}

fn should_suppress_repeat(
    last_triggered: Option<TriggeredHotkey>,
    modifiers: u32,
    vk: u32,
    now: Instant,
) -> bool {
    matches!(
        last_triggered,
        Some(last)
            if last.modifiers == modifiers
                && last.vk == vk
                && matches!(
                    now.checked_duration_since(last.triggered_at),
                    Some(elapsed) if elapsed <= REPEAT_SUPPRESSION_WINDOW
                )
    )
}

unsafe extern "system" fn hook_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CLOSE => {
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

const KEY_DOWN_MASK: u16 = 0x8000;

/// Snapshot modifier bits from actual keyboard state during startup or restart.
fn modifier_snapshot_from_async_state() -> u32 {
    let mut mods = 0u32;
    if (unsafe { GetAsyncKeyState(VK_CONTROL as i32) } as u16 & KEY_DOWN_MASK) != 0 {
        mods |= MOD_CONTROL_BIT;
    }
    if (unsafe { GetAsyncKeyState(VK_MENU as i32) } as u16 & KEY_DOWN_MASK) != 0 {
        mods |= MOD_ALT_BIT;
    }
    if (unsafe { GetAsyncKeyState(VK_SHIFT as i32) } as u16 & KEY_DOWN_MASK) != 0 {
        mods |= MOD_SHIFT_BIT;
    }
    if (unsafe { GetAsyncKeyState(VK_LWIN as i32) } as u16 & KEY_DOWN_MASK) != 0
        || (unsafe { GetAsyncKeyState(VK_RWIN as i32) } as u16 & KEY_DOWN_MASK) != 0
    {
        mods |= MOD_WIN_BIT;
    }
    mods
}

unsafe extern "system" fn low_level_keyboard_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code != HC_ACTION as i32 {
        return CallNextHookEx(None, code, wparam, lparam);
    }

    let msg = wparam.0 as u32;
    let key_down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;

    let kbd = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
    let vk = kbd.vkCode;
    let is_up = (kbd.flags.0 & LLKHF_UP.0) != 0;

    if kbd.dwExtraInfo == PROBE_EXTRA_INFO && kbd.vkCode == u32::from(VK_F24_CODE) {
        PROBE_ACKED.store(true, Ordering::SeqCst);
        if let Ok(mut guard) = HOOK_STATE.lock() {
            if let Some(state) = guard.as_mut() {
                state.last_callback_at = Instant::now();
            }
        }
        return LRESULT(1);
    }

    let (action_to_run, current_mods) = {
        let mut guard = HOOK_STATE.lock().unwrap_or_else(|e| e.into_inner());
        let Some(ref mut state) = *guard else {
            return CallNextHookEx(None, code, wparam, lparam);
        };
        let now = Instant::now();
        state.last_callback_at = now;

        if is_modifier_key(vk) {
            update_modifier_state(state, vk, is_up);
            if is_up {
                state.last_triggered = None;
            }
            return CallNextHookEx(None, code, wparam, lparam);
        }

        if is_up {
            state.last_triggered = None;
            return CallNextHookEx(None, code, wparam, lparam);
        }

        if !key_down {
            return CallNextHookEx(None, code, wparam, lparam);
        }

        let current_mods = state.current_modifiers;

        let mut result = None;
        for (bind_mods, bind_vk, action_name) in &state.hotkeys {
            if *bind_vk == vk && (bind_mods & MOD_MASK) == current_mods {
                if should_suppress_repeat(state.last_triggered, *bind_mods, *bind_vk, now) {
                    return LRESULT(1);
                }
                state.last_triggered = Some(TriggeredHotkey {
                    modifiers: *bind_mods,
                    vk: *bind_vk,
                    triggered_at: now,
                });
                result = Some(action_name.clone());
                break;
            }
        }
        (result, current_mods)
    };

    // Debug: emit every key down (non-modifier) to frontend when RECTANGLEWIN_DEBUG_KEYS=1
    if key_down && !is_up && !is_modifier_key(vk) {
        if let Ok(guard) = DEBUG_APP.lock() {
            if let Some(ref handle) = *guard {
                let label = shortcut::format_shortcut(current_mods, vk);
                let is_hotkey = action_to_run.is_some();
                let payload = KeyInterceptedPayload {
                    vk,
                    modifiers: current_mods,
                    label: label.clone(),
                    is_hotkey,
                };
                let handle_for_closure = handle.clone();
                let _ = handle.run_on_main_thread(move || {
                    let _ = handle_for_closure.emit("key-intercepted", &payload);
                });
            }
        }
    }

    if let Some(name) = action_to_run {
        let hwnd_raw = crate::win32::get_foreground_window().map(|h| h.0 as usize);
        if !should_consume_hotkey(true, hwnd_raw) {
            return CallNextHookEx(None, code, wparam, lparam);
        }

        if let Ok(guard) = HOOK_TRIGGER.lock() {
            if let Some(trigger) = guard.as_ref() {
                trigger(name, hwnd_raw);
            }
        }
        return LRESULT(1);
    }

    CallNextHookEx(None, code, wparam, lparam)
}

/// Start the low-level keyboard hook. Call from main thread (e.g. in setup).
/// `trigger` is called when a hotkey is pressed with (action_name, foreground_hwnd_at_press).
/// If `debug_app` is Some (dev build or RECTANGLEWIN_DEBUG_KEYS=1), every key down is emitted to the frontend.
pub fn start(
    hotkeys: Vec<HotkeyBinding>,
    trigger: Box<dyn Fn(String, Option<usize>) + Send>,
    debug_app: Option<tauri::AppHandle>,
) {
    if RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }

    if let Ok(mut guard) = DEBUG_APP.lock() {
        *guard = debug_app;
    }

    let bindings: Vec<(u32, u32, String)> = hotkeys
        .into_iter()
        .map(|b| (b.modifiers & MOD_MASK, b.virtual_key, b.action))
        .collect();

    {
        let mut guard = HOOK_TRIGGER.lock().unwrap_or_else(|e| e.into_inner());
        *guard = Some(trigger);
    }
    {
        let mut state = HOOK_STATE.lock().unwrap_or_else(|e| e.into_inner());
        *state = Some(HookState {
            hotkeys: bindings,
            last_triggered: None,
            current_modifiers: modifier_snapshot_from_async_state(),
            last_callback_at: Instant::now(),
        });
    }

    thread::spawn(|| {
        struct RunningGuard;

        impl Drop for RunningGuard {
            fn drop(&mut self) {
                HOOK_WINDOW.store(0, Ordering::SeqCst);
                RUNNING.store(false, Ordering::SeqCst);
            }
        }

        let _guard = RunningGuard;

        unsafe {
            // STA thread like C# HotkeyManager - required for hook delivery on some setups
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

            let Ok(hmod) = GetModuleHandleW(None) else {
                return;
            };
            let hinstance = std::mem::transmute::<_, windows::Win32::Foundation::HINSTANCE>(hmod);

            let class_name = w!("RectangleWinHook");
            let mut wc = WNDCLASSW::default();
            wc.lpfnWndProc = Some(hook_wndproc);
            wc.hInstance = hinstance;
            wc.lpszClassName = class_name;
            let _ = RegisterClassW(&wc);

            let Ok(hwnd) = CreateWindowExW(
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
            ) else {
                return;
            };

            HOOK_WINDOW.store(hwnd.0 as isize, Ordering::SeqCst);

            // Pass module handle like C# (GetHINSTANCE(Module)); NULL can fail in some loaders
            let Ok(hook) =
                SetWindowsHookExW(WH_KEYBOARD_LL, Some(low_level_keyboard_proc), hinstance, 0)
            else {
                let _ = DestroyWindow(hwnd);
                return;
            };

            let mut msg = std::mem::zeroed();
            while GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            let _ = UnhookWindowsHookEx(hook);
        }
    });
}

/// Update the hotkey list (e.g. after config reload). Hook thread keeps running.
pub fn set_hotkeys(hotkeys: Vec<HotkeyBinding>) {
    let bindings: Vec<(u32, u32, String)> = hotkeys
        .into_iter()
        .map(|b| (b.modifiers & MOD_MASK, b.virtual_key, b.action))
        .collect();

    if let Ok(mut state) = HOOK_STATE.lock() {
        if let Some(s) = state.as_mut() {
            s.hotkeys = bindings;
            s.last_triggered = None;
            s.current_modifiers = modifier_snapshot_from_async_state();
        }
    }
}

pub fn is_running() -> bool {
    RUNNING.load(Ordering::SeqCst)
}

pub fn stop(timeout: Duration) -> bool {
    if !RUNNING.load(Ordering::SeqCst) {
        return true;
    }

    let hwnd_raw = HOOK_WINDOW.load(Ordering::SeqCst);
    if hwnd_raw == 0 {
        return false;
    }

    let hwnd = HWND(hwnd_raw as *mut std::ffi::c_void);
    let _ = unsafe { PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0)) };

    let deadline = Instant::now() + timeout;
    while RUNNING.load(Ordering::SeqCst) && Instant::now() < deadline {
        thread::sleep(STOP_WAIT_SLICE);
    }

    !RUNNING.load(Ordering::SeqCst)
}

pub fn probe(timeout: Duration) -> bool {
    if !RUNNING.load(Ordering::SeqCst) {
        return false;
    }

    PROBE_ACKED.store(false, Ordering::SeqCst);

    let inputs = [
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(VK_F24_CODE),
                    wScan: 0,
                    dwFlags: Default::default(),
                    time: 0,
                    dwExtraInfo: PROBE_EXTRA_INFO,
                },
            },
        },
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(VK_F24_CODE),
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: PROBE_EXTRA_INFO,
                },
            },
        },
    ];

    let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
    if sent != inputs.len() as u32 {
        return false;
    }

    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if PROBE_ACKED.load(Ordering::SeqCst) {
            return true;
        }
        thread::sleep(PROBE_WAIT_SLICE);
    }

    false
}

#[cfg(test)]
mod tests {
    use super::{
        modifier_bit_for_vk, should_consume_hotkey, should_suppress_repeat, update_modifier_state,
        HookState, TriggeredHotkey, MOD_ALT_BIT, MOD_CONTROL_BIT, MOD_SHIFT_BIT, MOD_WIN_BIT,
        REPEAT_SUPPRESSION_WINDOW, VK_CONTROL, VK_LMENU, VK_LSHIFT, VK_LWIN,
    };
    use std::time::{Duration, Instant};

    #[test]
    fn consumes_only_when_a_real_target_window_exists() {
        assert!(!should_consume_hotkey(false, Some(123)));
        assert!(!should_consume_hotkey(true, None));
        assert!(should_consume_hotkey(true, Some(123)));
    }

    #[test]
    fn repeat_suppression_blocks_only_short_repeat_window() {
        let now = Instant::now();
        let last_triggered = Some(TriggeredHotkey {
            modifiers: 0x09,
            vk: 0x25,
            triggered_at: now,
        });

        assert!(should_suppress_repeat(
            last_triggered,
            0x09,
            0x25,
            now + Duration::from_millis(250)
        ));
        assert!(!should_suppress_repeat(
            last_triggered,
            0x09,
            0x25,
            now + REPEAT_SUPPRESSION_WINDOW + Duration::from_millis(1)
        ));
        assert!(!should_suppress_repeat(
            last_triggered,
            0x08,
            0x25,
            now + Duration::from_millis(250)
        ));
        assert!(!should_suppress_repeat(
            last_triggered,
            0x09,
            0x27,
            now + Duration::from_millis(250)
        ));
    }

    #[test]
    fn modifier_vk_mapping_covers_supported_keys() {
        assert_eq!(modifier_bit_for_vk(VK_LWIN), Some(MOD_WIN_BIT));
        assert_eq!(modifier_bit_for_vk(VK_CONTROL), Some(MOD_CONTROL_BIT));
        assert_eq!(modifier_bit_for_vk(VK_LMENU), Some(MOD_ALT_BIT));
        assert_eq!(modifier_bit_for_vk(VK_LSHIFT), Some(MOD_SHIFT_BIT));
        assert_eq!(modifier_bit_for_vk(0x41), None);
    }

    #[test]
    fn modifier_state_tracks_press_and_release() {
        let mut state = HookState {
            hotkeys: Vec::new(),
            last_triggered: None,
            current_modifiers: 0,
            last_callback_at: Instant::now(),
        };

        update_modifier_state(&mut state, VK_LWIN, false);
        update_modifier_state(&mut state, VK_CONTROL, false);
        assert_eq!(state.current_modifiers, MOD_WIN_BIT | MOD_CONTROL_BIT);

        update_modifier_state(&mut state, VK_LWIN, true);
        assert_eq!(state.current_modifiers, MOD_CONTROL_BIT);

        update_modifier_state(&mut state, VK_CONTROL, true);
        assert_eq!(state.current_modifiers, 0);
    }
}
