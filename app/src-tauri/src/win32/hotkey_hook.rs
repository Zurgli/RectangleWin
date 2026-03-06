//! Low-level keyboard hook (WH_KEYBOARD_LL) so we can capture Win+key and override
//! any other shortcuts. Runs on a dedicated thread with a hidden window and message loop.

use crate::config::HotkeyBinding;
use crate::shortcut;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Emitter;
use std::sync::Mutex;
use std::thread;
use windows::core::w;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Com::CoInitializeEx;
use windows::Win32::System::Com::COINIT_APARTMENTTHREADED;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW,
    RegisterClassW, SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx,
    HC_ACTION, KBDLLHOOKSTRUCT, LLKHF_UP, WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
    WNDCLASSW, WS_EX_TOOLWINDOW, WS_OVERLAPPED,
};

const MOD_MASK: u32 = 0x000F; // MOD_ALT | MOD_CONTROL | MOD_SHIFT | MOD_WIN
const VK_LWIN: u32 = 0x5B;
const VK_RWIN: u32 = 0x5C;
const VK_CONTROL: u32 = 0x11;
const VK_MENU: u32 = 0x12;   // generic Alt
const VK_SHIFT: u32 = 0x10;
const VK_LMENU: u32 = 0xA4;  // left Alt (hook can receive these instead of VK_MENU)
const VK_RMENU: u32 = 0xA5;
const VK_LCONTROL: u32 = 0xA2;
const VK_RCONTROL: u32 = 0xA3;
const VK_LSHIFT: u32 = 0xA0;
const VK_RSHIFT: u32 = 0xA1;

static HOOK_STATE: Mutex<Option<HookState>> = Mutex::new(None);
/// usize = HWND.0 for passing across threads (HWND is !Send).
static HOOK_TRIGGER: Mutex<Option<Box<dyn Fn(String, Option<usize>) + Send>>> = Mutex::new(None);
/// When set (e.g. RECTANGLEWIN_DEBUG_KEYS=1), every intercepted key down is emitted to the frontend.
static DEBUG_APP: Mutex<Option<tauri::AppHandle>> = Mutex::new(None);
static RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(serde::Serialize)]
struct KeyInterceptedPayload {
    vk: u32,
    modifiers: u32,
    label: String,
    is_hotkey: bool,
}

struct HookState {
    hotkeys: Vec<(u32, u32, String)>, // (modifiers & MOD_MASK, vk, action_name)
    last_triggered: Option<(u32, u32)>, // (modifiers, vk) to avoid repeat
}

fn is_modifier_key(vk: u32) -> bool {
    matches!(
        vk,
        VK_LWIN | VK_RWIN
            | VK_CONTROL | VK_LCONTROL | VK_RCONTROL
            | VK_MENU | VK_LMENU | VK_RMENU
            | VK_SHIFT | VK_LSHIFT | VK_RSHIFT
    )
}

unsafe extern "system" fn hook_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

const KEY_DOWN_MASK: u16 = 0x8000;

/// Build current modifier bits from actual keyboard state (matches C# HotkeyManager).
fn current_modifiers_from_async_state() -> u32 {
    let mut mods = 0u32;
    if (unsafe { GetAsyncKeyState(VK_CONTROL as i32) } as u16 & KEY_DOWN_MASK) != 0 {
        mods |= 0x02; // MOD_CONTROL
    }
    if (unsafe { GetAsyncKeyState(VK_MENU as i32) } as u16 & KEY_DOWN_MASK) != 0 {
        mods |= 0x01; // MOD_ALT
    }
    if (unsafe { GetAsyncKeyState(VK_SHIFT as i32) } as u16 & KEY_DOWN_MASK) != 0 {
        mods |= 0x04; // MOD_SHIFT
    }
    if (unsafe { GetAsyncKeyState(VK_LWIN as i32) } as u16 & KEY_DOWN_MASK) != 0
        || (unsafe { GetAsyncKeyState(VK_RWIN as i32) } as u16 & KEY_DOWN_MASK) != 0
    {
        mods |= 0x08; // MOD_WIN
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

    let action_to_run = {
        let mut guard = HOOK_STATE.lock().unwrap_or_else(|e| e.into_inner());
        let Some(ref mut state) = *guard else {
            return CallNextHookEx(None, code, wparam, lparam);
        };

        if is_modifier_key(vk) {
            return CallNextHookEx(None, code, wparam, lparam);
        }

        if is_up {
            state.last_triggered = None;
            return CallNextHookEx(None, code, wparam, lparam);
        }

        if !key_down {
            return CallNextHookEx(None, code, wparam, lparam);
        }

        let current_mods = current_modifiers_from_async_state();

        let mut result = None;
        for (bind_mods, bind_vk, action_name) in &state.hotkeys {
            if *bind_vk == vk && (bind_mods & MOD_MASK) == current_mods {
                if state.last_triggered == Some((*bind_mods, *bind_vk)) {
                    return LRESULT(1);
                }
                state.last_triggered = Some((*bind_mods, *bind_vk));
                result = Some(action_name.clone());
                break;
            }
        }
        result
    };

    // Debug: emit every key down (non-modifier) to frontend when RECTANGLEWIN_DEBUG_KEYS=1
    if key_down && !is_up && !is_modifier_key(vk) {
        if let Ok(guard) = DEBUG_APP.lock() {
            if let Some(ref handle) = *guard {
                let mods = current_modifiers_from_async_state();
                let label = shortcut::format_shortcut(mods, vk);
                let is_hotkey = action_to_run.is_some();
                let payload = KeyInterceptedPayload {
                    vk,
                    modifiers: mods,
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
        if let Ok(guard) = HOOK_TRIGGER.lock() {
            if let Some(trigger) = guard.as_ref() {
                let hwnd_raw = crate::win32::get_foreground_window().map(|h| h.0 as usize);
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
        });
    }

    thread::spawn(|| {
        unsafe {
            // STA thread like C# HotkeyManager - required for hook delivery on some setups
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

            let hmod = GetModuleHandleW(None).expect("GetModuleHandleW");
            let hinstance = std::mem::transmute::<_, windows::Win32::Foundation::HINSTANCE>(hmod);

            let class_name = w!("RectangleWinHook");
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
            .expect("CreateWindowExW hook window");

            // Pass module handle like C# (GetHINSTANCE(Module)); NULL can fail in some loaders
            let hook = SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(low_level_keyboard_proc),
                hinstance,
                0,
            )
            .expect("SetWindowsHookExW WH_KEYBOARD_LL");

            let mut msg = std::mem::zeroed();
            while GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            let _ = UnhookWindowsHookEx(hook);
        }
        RUNNING.store(false, Ordering::SeqCst);
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
        }
    }
}
