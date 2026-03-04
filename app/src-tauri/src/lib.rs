mod config;
mod engine;
mod manager;
mod rect;
mod shortcut;
#[cfg(windows)]
mod win32;

use std::str::FromStr;
use config::{config_from_frontend, load as config_load, save as config_save, Config, ConfigForFrontend};
use manager::{ExecuteOptions, WindowManager};
use std::sync::Mutex;
use tauri::Manager;

pub struct AppState {
    pub config: Mutex<Config>,
    pub manager: Mutex<WindowManager>,
    /// Map plugin shortcut string (e.g. "Alt+Super+Left") to action name.
    pub shortcut_to_action: Mutex<std::collections::HashMap<String, String>>,
}

fn shortcut_to_plugin_format(s: &str) -> String {
    s.replace("Win+", "Super+").replace("Win", "Super")
}

#[tauri::command]
fn get_config_path() -> String {
    config::config_path().display().to_string()
}

#[tauri::command]
fn load_config(state: tauri::State<AppState>) -> Result<ConfigForFrontend, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.to_frontend())
}

#[tauri::command]
fn save_config(state: tauri::State<AppState>, payload: ConfigForFrontend) -> Result<ConfigForFrontend, String> {
    let new_config = config_from_frontend(payload);
    config_save(&new_config).map_err(|e| e.to_string())?;
    {
        let mut config = state.config.lock().map_err(|e| e.to_string())?;
        *config = new_config.clone();
    }
    {
        let mut map = state.shortcut_to_action.lock().map_err(|e| e.to_string())?;
        *map = build_shortcut_map(&new_config);
    }
    #[cfg(windows)]
    win32::set_hotkeys(new_config.hotkeys.clone());
    Ok(new_config.to_frontend())
}

#[tauri::command]
fn reload_config(state: tauri::State<AppState>) -> Result<ConfigForFrontend, String> {
    let new_config = config_load();
    {
        let mut config = state.config.lock().map_err(|e| e.to_string())?;
        *config = new_config.clone();
    }
    {
        let mut map = state.shortcut_to_action.lock().map_err(|e| e.to_string())?;
        *map = build_shortcut_map(&new_config);
    }
    #[cfg(windows)]
    win32::set_hotkeys(new_config.hotkeys.clone());
    Ok(new_config.to_frontend())
}

#[tauri::command]
fn revert_to_defaults(state: tauri::State<AppState>) -> Result<ConfigForFrontend, String> {
    let default_config = Config::default();
    config_save(&default_config).map_err(|e| e.to_string())?;
    {
        let mut config = state.config.lock().map_err(|e| e.to_string())?;
        *config = default_config.clone();
    }
    {
        let mut map = state.shortcut_to_action.lock().map_err(|e| e.to_string())?;
        *map = build_shortcut_map(&default_config);
    }
    #[cfg(windows)]
    win32::set_hotkeys(default_config.hotkeys.clone());
    Ok(default_config.to_frontend())
}

fn build_shortcut_map(config: &Config) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    #[cfg(windows)]
    {
        use tauri_plugin_global_shortcut::Shortcut;
        for b in &config.hotkeys {
            let s = shortcut::format_shortcut(b.modifiers, b.virtual_key);
            let plugin_fmt = shortcut_to_plugin_format(&s);
            if let Ok(shortcut) = Shortcut::from_str(&plugin_fmt) {
                map.insert(shortcut.to_string(), b.action.clone());
            }
        }
    }
    #[cfg(not(windows))]
    {
        for b in &config.hotkeys {
            let s = shortcut::format_shortcut(b.modifiers, b.virtual_key);
            let plugin_fmt = shortcut_to_plugin_format(&s);
            map.insert(plugin_fmt, b.action.clone());
        }
    }
    map
}

#[cfg(windows)]
fn register_hotkeys(app: &tauri::AppHandle, config: &Config) -> Result<(), String> {
    let handle = app.clone();
    let hotkeys = config.hotkeys.clone();
    // HWND is not Send; pass as usize and convert back on main thread
    let trigger: Box<dyn Fn(String, Option<usize>) + Send> = Box::new(
        move |action_name: String, hwnd_raw: Option<usize>| {
            let value = handle.clone();
            let _ = handle.run_on_main_thread(move || {
                if let Some(action) = engine::WindowAction::from_str(&action_name) {
                    let options = {
                        let state = value.state::<AppState>();
                        state
                            .config
                            .lock()
                            .ok()
                            .map(|c| ExecuteOptions::from(&*c))
                            .unwrap_or_default()
                    };
                    let hwnd_override = hwnd_raw
                        .map(|p| windows::Win32::Foundation::HWND(p as *mut std::ffi::c_void));
                    let _ = value
                        .state::<AppState>()
                        .manager
                        .lock()
                        .ok()
                        .map(|mut m| m.execute(action, hwnd_override, &options))
                        .unwrap_or(false);
                }
            });
        },
    );
    let debug_app = std::env::var_os("RECTANGLEWIN_DEBUG_KEYS")
        .filter(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .map(|_| app.clone());
    win32::start_lowlevel_hook(hotkeys, trigger, debug_app);
    Ok(())
}

#[cfg(not(windows))]
fn register_hotkeys(_app: &tauri::AppHandle, _config: &Config) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
fn exit_app() {
    std::process::exit(0);
}

#[tauri::command]
fn open_config_in_editor() -> Result<(), String> {
    let path = config::config_path();
    if !path.exists() {
        let _ = std::fs::write(&path, "{}").map_err(|e| e.to_string())?;
    }
    opener::open(path).map_err(|e| e.to_string())
}

#[tauri::command]
fn open_config_file_location() -> Result<(), String> {
    let path = config::config_path();
    let dir = path.parent().ok_or_else(|| "No parent directory".to_string())?;
    opener::open(dir).map_err(|e| e.to_string())
}

#[tauri::command]
fn run_action(state: tauri::State<AppState>, action: String) -> Result<bool, String> {
    let action = engine::WindowAction::from_str(&action).ok_or_else(|| "Unknown action".to_string())?;
    let options = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        ExecuteOptions::from(&*config)
    };
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.execute(action, None, &options))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let initial_config = config_load();
    let shortcut_map = build_shortcut_map(&initial_config);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .manage(AppState {
            config: Mutex::new(initial_config.clone()),
            manager: Mutex::new(WindowManager::new()),
            shortcut_to_action: Mutex::new(shortcut_map),
        })
        .setup(move |app| {
            #[cfg(windows)]
            register_hotkeys(&app.handle(), &initial_config).ok();

            // Tray: single left-click shows window; right-click shows menu with Quit
            let handle = app.handle().clone();
            const TRAY_ICON: tauri::image::Image<'static> = tauri::include_image!("icons/icon.ico");
            let quit_item = tauri::menu::MenuItem::with_id(&handle, "quit", "Quit RectangleWin", true, None::<&str>)
                .expect("tray quit menu item");
            let menu = tauri::menu::Menu::with_items(&handle, &[&quit_item]).expect("tray menu");
            let h = handle.clone();
            let _ = tauri::tray::TrayIconBuilder::new()
                .icon(TRAY_ICON)
                .tooltip("RectangleWin")
                .menu(&menu)
                .on_menu_event(move |_app, event| {
                    if event.id().as_ref() == "quit" {
                        std::process::exit(0);
                    }
                })
                .on_tray_icon_event(move |_tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { button, .. } = event {
                        if matches!(button, tauri::tray::MouseButton::Left) {
                            if let Some(w) = h.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                    }
                })
                .build(&handle);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config_path,
            load_config,
            save_config,
            reload_config,
            revert_to_defaults,
            exit_app,
            open_config_in_editor,
            open_config_file_location,
            run_action,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
