//! Config load/save for %LocalAppData%\RectangleWin\config.json.
//! Persisted shape matches C# AppConfig.PersistedConfig for compatibility.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::shortcut::{format_shortcut, try_parse_shortcut, MOD_ALT, MOD_NOREPEAT, MOD_WIN};

/// Runtime hotkey binding (action + modifiers + vk).
#[derive(Clone, Debug)]
pub struct HotkeyBinding {
    pub action: String,
    pub modifiers: u32,
    pub virtual_key: u32,
}

/// Persisted config (only these fields are read/written to JSON).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedConfig {
    #[serde(default = "default_launch_on_login")]
    pub launch_on_login: bool,
    #[serde(default)]
    pub gap_size: f32,
    #[serde(default)]
    pub screen_edge_gap_top: f32,
    #[serde(default)]
    pub screen_edge_gap_bottom: f32,
    #[serde(default)]
    pub screen_edge_gap_left: f32,
    #[serde(default)]
    pub screen_edge_gap_right: f32,
    #[serde(default)]
    pub screen_edge_gaps_on_main_screen_only: bool,
    #[serde(default)]
    pub taskbar_gap_compensation: i32,
    #[serde(default)]
    pub taskbar_gap_compensation_left: i32,
    #[serde(default)]
    pub taskbar_gap_compensation_right: i32,
    #[serde(default = "default_true")]
    pub apply_gaps_to_maximize: bool,
    #[serde(default = "default_true")]
    pub apply_gaps_to_maximize_height: bool,
    #[serde(default)]
    pub hotkeys: Vec<PersistedHotkey>,
    #[serde(default = "default_thirds_layout")]
    pub thirds_layout: String,
}

fn default_launch_on_login() -> bool {
    true
}
fn default_true() -> bool {
    true
}
fn default_thirds_layout() -> String {
    "Thirds".into()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistedHotkey {
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shortcut: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifiers: Option<u32>,
    #[serde(rename = "virtualKey", skip_serializing_if = "Option::is_none")]
    pub virtual_key: Option<u32>,
}

/// Full app config (persisted + runtime defaults).
#[derive(Clone, Debug)]
pub struct Config {
    pub launch_on_login: bool,
    pub gap_size: f32,
    pub screen_edge_gap_top: f32,
    pub screen_edge_gap_bottom: f32,
    pub screen_edge_gap_left: f32,
    pub screen_edge_gap_right: f32,
    pub screen_edge_gaps_on_main_screen_only: bool,
    pub taskbar_gap_compensation: i32,
    pub taskbar_gap_compensation_left: i32,
    pub taskbar_gap_compensation_right: i32,
    pub apply_gaps_to_maximize: bool,
    pub apply_gaps_to_maximize_height: bool,
    pub hotkeys: Vec<HotkeyBinding>,
    pub thirds_layout: String,
}

/// Config shape for frontend (camelCase, hotkeys with action + shortcut string).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigForFrontend {
    pub launch_on_login: bool,
    pub gap_size: f32,
    pub screen_edge_gap_top: f32,
    pub screen_edge_gap_bottom: f32,
    pub screen_edge_gap_left: f32,
    pub screen_edge_gap_right: f32,
    pub screen_edge_gaps_on_main_screen_only: bool,
    pub taskbar_gap_compensation: i32,
    pub taskbar_gap_compensation_left: i32,
    pub taskbar_gap_compensation_right: i32,
    pub apply_gaps_to_maximize: bool,
    pub apply_gaps_to_maximize_height: bool,
    pub thirds_layout: String,
    pub hotkeys: Vec<HotkeyForFrontend>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyForFrontend {
    pub action: String,
    pub shortcut: String,
}

impl Config {
    pub fn to_frontend(&self) -> ConfigForFrontend {
        ConfigForFrontend {
            launch_on_login: self.launch_on_login,
            gap_size: self.gap_size,
            screen_edge_gap_top: self.screen_edge_gap_top,
            screen_edge_gap_bottom: self.screen_edge_gap_bottom,
            screen_edge_gap_left: self.screen_edge_gap_left,
            screen_edge_gap_right: self.screen_edge_gap_right,
            screen_edge_gaps_on_main_screen_only: self.screen_edge_gaps_on_main_screen_only,
            taskbar_gap_compensation: self.taskbar_gap_compensation,
            taskbar_gap_compensation_left: self.taskbar_gap_compensation_left,
            taskbar_gap_compensation_right: self.taskbar_gap_compensation_right,
            apply_gaps_to_maximize: self.apply_gaps_to_maximize,
            apply_gaps_to_maximize_height: self.apply_gaps_to_maximize_height,
            thirds_layout: self.thirds_layout.clone(),
            hotkeys: self
                .hotkeys
                .iter()
                .map(|b| HotkeyForFrontend {
                    action: b.action.clone(),
                    shortcut: format_shortcut(b.modifiers, b.virtual_key),
                })
                .collect(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::default_hotkeys()
    }
}

impl Config {
    fn default_hotkeys() -> Self {
        // Win+Alt: low-level hook captures these so we override OS and other apps.
        let win_alt = MOD_NOREPEAT | MOD_WIN | MOD_ALT;
        let hotkeys = vec![
            ("LeftHalf", 0x25),
            ("RightHalf", 0x27),
            ("TopHalf", 0x26),
            ("BottomHalf", 0x28),
            ("UpperLeft", 0x55),
            ("UpperRight", 0x49),
            ("LowerLeft", 0x4A),
            ("LowerRight", 0x4B),
            ("Maximize", 0x0D),
            ("Center", 0x43),
            ("Undo", 0x2E),
            ("FirstThird", 0x44),
            ("FirstTwoThirds", 0x45),
            ("CenterThird", 0x46),
            ("LastTwoThirds", 0x54),
            ("LastThird", 0x47),
            ("NextDisplay", 0x4E),
            ("PreviousDisplay", 0x50),
        ]
        .into_iter()
        .map(|(action, vk)| HotkeyBinding {
            action: action.into(),
            modifiers: win_alt,
            virtual_key: vk,
        })
        .collect();

        Self {
            launch_on_login: true,
            gap_size: -2.0,
            screen_edge_gap_top: 0.0,
            screen_edge_gap_bottom: 0.0,
            screen_edge_gap_left: 0.0,
            screen_edge_gap_right: 0.0,
            screen_edge_gaps_on_main_screen_only: false,
            taskbar_gap_compensation: 0,
            taskbar_gap_compensation_left: 0,
            taskbar_gap_compensation_right: 0,
            apply_gaps_to_maximize: true,
            apply_gaps_to_maximize_height: true,
            hotkeys,
            thirds_layout: "Thirds".into(),
        }
    }
}

pub fn config_path() -> PathBuf {
    let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".into());
    let dir = PathBuf::from(local_app_data).join("RectangleWin");
    let _ = fs::create_dir_all(&dir);
    dir.join("config.json")
}

pub fn load() -> Config {
    let path = config_path();
    let default_config = Config::default();
    if !path.exists() {
        return default_config;
    }
    let Ok(json) = fs::read_to_string(&path) else {
        return default_config;
    };
    let p: PersistedConfig = match serde_json::from_str(&json) {
        Ok(x) => x,
        Err(_) => return default_config,
    };

    let thirds_layout = if p.thirds_layout.eq_ignore_ascii_case("Fifths") {
        "Fifths".into()
    } else if p.thirds_layout.eq_ignore_ascii_case("Fourths") {
        "Fourths".into()
    } else {
        "Thirds".into()
    };

    let hotkeys: Vec<HotkeyBinding> = if p.hotkeys.is_empty() {
        default_config.hotkeys
    } else {
        p.hotkeys
            .into_iter()
            .filter_map(|h| {
                let action = if h.action.eq_ignore_ascii_case("Restore") {
                    "Undo".into()
                } else if h.action.is_empty() {
                    return None;
                } else {
                    h.action
                };
                let (modifiers, virtual_key) = if let Some(ref s) = h.shortcut {
                    match try_parse_shortcut(s) {
                        Some((m, v)) => (m, v),
                        None => return None,
                    }
                } else if let (Some(mods), Some(vk)) = (h.modifiers, h.virtual_key) {
                    (mods | MOD_NOREPEAT, vk)
                } else {
                    return None;
                };
                Some(HotkeyBinding {
                    action,
                    modifiers,
                    virtual_key,
                })
            })
            .collect()
    };

    let hotkeys = if hotkeys.is_empty() {
        Config::default().hotkeys
    } else {
        hotkeys
    };

    Config {
        launch_on_login: p.launch_on_login,
        gap_size: p.gap_size,
        screen_edge_gap_top: p.screen_edge_gap_top,
        screen_edge_gap_bottom: p.screen_edge_gap_bottom,
        screen_edge_gap_left: p.screen_edge_gap_left,
        screen_edge_gap_right: p.screen_edge_gap_right,
        screen_edge_gaps_on_main_screen_only: p.screen_edge_gaps_on_main_screen_only,
        taskbar_gap_compensation: p.taskbar_gap_compensation,
        taskbar_gap_compensation_left: p.taskbar_gap_compensation_left,
        taskbar_gap_compensation_right: p.taskbar_gap_compensation_right,
        apply_gaps_to_maximize: p.apply_gaps_to_maximize,
        apply_gaps_to_maximize_height: p.apply_gaps_to_maximize_height,
        hotkeys,
        thirds_layout,
    }
}

/// Build Config from frontend payload (e.g. after user edits in UI).
pub fn config_from_frontend(p: ConfigForFrontend) -> Config {
    let hotkeys: Vec<HotkeyBinding> = p
        .hotkeys
        .into_iter()
        .filter_map(|h| {
            let action = if h.action.eq_ignore_ascii_case("Restore") {
                "Undo".into()
            } else if h.action.is_empty() {
                return None;
            } else {
                h.action
            };
            let (modifiers, virtual_key) = try_parse_shortcut(h.shortcut.trim())?;
            Some(HotkeyBinding {
                action,
                modifiers,
                virtual_key,
            })
        })
        .collect();
    let hotkeys = if hotkeys.is_empty() {
        Config::default().hotkeys
    } else {
        hotkeys
    };
    Config {
        launch_on_login: p.launch_on_login,
        gap_size: p.gap_size,
        screen_edge_gap_top: p.screen_edge_gap_top,
        screen_edge_gap_bottom: p.screen_edge_gap_bottom,
        screen_edge_gap_left: p.screen_edge_gap_left,
        screen_edge_gap_right: p.screen_edge_gap_right,
        screen_edge_gaps_on_main_screen_only: p.screen_edge_gaps_on_main_screen_only,
        taskbar_gap_compensation: p.taskbar_gap_compensation,
        taskbar_gap_compensation_left: p.taskbar_gap_compensation_left,
        taskbar_gap_compensation_right: p.taskbar_gap_compensation_right,
        apply_gaps_to_maximize: p.apply_gaps_to_maximize,
        apply_gaps_to_maximize_height: p.apply_gaps_to_maximize_height,
        hotkeys,
        thirds_layout: if p.thirds_layout.eq_ignore_ascii_case("Fifths") {
            "Fifths".into()
        } else if p.thirds_layout.eq_ignore_ascii_case("Fourths") {
            "Fourths".into()
        } else {
            "Thirds".into()
        },
    }
}

pub fn save(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let path = config_path();
    let p = PersistedConfig {
        launch_on_login: config.launch_on_login,
        gap_size: config.gap_size,
        screen_edge_gap_top: config.screen_edge_gap_top,
        screen_edge_gap_bottom: config.screen_edge_gap_bottom,
        screen_edge_gap_left: config.screen_edge_gap_left,
        screen_edge_gap_right: config.screen_edge_gap_right,
        screen_edge_gaps_on_main_screen_only: config.screen_edge_gaps_on_main_screen_only,
        taskbar_gap_compensation: config.taskbar_gap_compensation,
        taskbar_gap_compensation_left: config.taskbar_gap_compensation_left,
        taskbar_gap_compensation_right: config.taskbar_gap_compensation_right,
        apply_gaps_to_maximize: config.apply_gaps_to_maximize,
        apply_gaps_to_maximize_height: config.apply_gaps_to_maximize_height,
        hotkeys: config
            .hotkeys
            .iter()
            .map(|b| PersistedHotkey {
                action: if b.action == "Undo" {
                    "Undo".into()
                } else {
                    b.action.clone()
                },
                shortcut: Some(format_shortcut(b.modifiers, b.virtual_key)),
                modifiers: None,
                virtual_key: None,
            })
            .collect(),
        thirds_layout: config.thirds_layout.clone(),
    };
    let json = serde_json::to_string_pretty(&p)?;
    fs::write(path, json)?;
    Ok(())
}
