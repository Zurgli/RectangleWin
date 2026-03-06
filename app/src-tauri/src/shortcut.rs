//! Parse and format human-readable shortcuts (e.g. "Win+Alt+Left") for config and UI.
//! Matches C# ShortcutHelper behavior.

pub const MOD_ALT: u32 = 0x0001;
pub const MOD_CONTROL: u32 = 0x0002;
pub const MOD_SHIFT: u32 = 0x0004;
pub const MOD_WIN: u32 = 0x0008;
pub const MOD_NOREPEAT: u32 = 0x4000;

/// Parse modifier name to flag.
fn parse_modifier(s: &str) -> Option<u32> {
    match s.to_lowercase().as_str() {
        "win" => Some(MOD_WIN),
        "alt" => Some(MOD_ALT),
        "ctrl" | "control" => Some(MOD_CONTROL),
        "shift" => Some(MOD_SHIFT),
        _ => None,
    }
}

/// Virtual key code to display name.
pub fn vk_to_key_name(vk: u32) -> String {
    match vk {
        0x25 => "Left".into(),
        0x26 => "Up".into(),
        0x27 => "Right".into(),
        0x28 => "Down".into(),
        0x0D => "Enter".into(),
        0x2E => "Delete".into(),
        0x30 => "0".into(),
        0x31 => "1".into(),
        0x32 => "2".into(),
        0x33 => "3".into(),
        0x34 => "4".into(),
        0x35 => "5".into(),
        0x36 => "6".into(),
        0x37 => "7".into(),
        0x38 => "8".into(),
        0x39 => "9".into(),
        0x41 => "A".into(),
        0x42 => "B".into(),
        0x43 => "C".into(),
        0x44 => "D".into(),
        0x45 => "E".into(),
        0x46 => "F".into(),
        0x47 => "G".into(),
        0x48 => "H".into(),
        0x49 => "I".into(),
        0x4A => "J".into(),
        0x4B => "K".into(),
        0x4E => "N".into(),
        0x50 => "P".into(),
        0x51 => "Q".into(),
        0x52 => "R".into(),
        0x53 => "S".into(),
        0x54 => "T".into(),
        0x55 => "U".into(),
        0x57 => "W".into(),
        _ => format!("0x{vk:X}"),
    }
}

fn key_name_to_vk(name: &str) -> Option<u32> {
    let n = name.trim();
    if n.is_empty() {
        return None;
    }
    if n.len() == 1 {
        let c = n.chars().next()?;
        if ('0'..='9').contains(&c) {
            return Some(0x30 + (c as u32 - '0' as u32));
        }
        if ('A'..='Z').contains(&c) {
            return Some(0x41 + (c as u32 - 'A' as u32));
        }
        if ('a'..='z').contains(&c) {
            return Some(0x41 + (c as u32 - 'a' as u32));
        }
    }
    let vk = match n.to_lowercase().as_str() {
        "left" => 0x25,
        "up" => 0x26,
        "right" => 0x27,
        "down" => 0x28,
        "enter" => 0x0D,
        "delete" => 0x2E,
        "1" => 0x31,
        "2" => 0x32,
        "3" => 0x33,
        "4" => 0x34,
        "5" => 0x35,
        "6" => 0x36,
        "a" => 0x41,
        "b" => 0x42,
        "c" => 0x43,
        "d" => 0x44,
        "e" => 0x45,
        "f" => 0x46,
        "g" => 0x47,
        "h" => 0x48,
        "i" => 0x49,
        "j" => 0x4A,
        "k" => 0x4B,
        "n" => 0x4E,
        "p" => 0x50,
        "q" => 0x51,
        "r" => 0x52,
        "s" => 0x53,
        "t" => 0x54,
        "u" => 0x55,
        "w" => 0x57,
        _ => return None,
    };
    Some(vk)
}

/// Format (modifiers, vk) to string like "Win+Alt+Left".
pub fn format_shortcut(modifiers: u32, vk: u32) -> String {
    let mut parts: Vec<String> = Vec::new();
    if (modifiers & MOD_WIN) != 0 {
        parts.push("Win".into());
    }
    if (modifiers & MOD_ALT) != 0 {
        parts.push("Alt".into());
    }
    if (modifiers & MOD_CONTROL) != 0 {
        parts.push("Ctrl".into());
    }
    if (modifiers & MOD_SHIFT) != 0 {
        parts.push("Shift".into());
    }
    parts.push(vk_to_key_name(vk));
    parts.join("+")
}

/// Parse shortcut string like "Win+Alt+Left" into (modifiers | MOD_NOREPEAT, vk).
/// Returns None if parsing fails.
pub fn try_parse_shortcut(shortcut: &str) -> Option<(u32, u32)> {
    let shortcut = shortcut.trim();
    if shortcut.is_empty() {
        return None;
    }
    let parts: Vec<&str> = shortcut.split('+').map(|p| p.trim()).filter(|p| !p.is_empty()).collect();
    if parts.is_empty() {
        return None;
    }
    let key_name = parts.last()?;
    let mut modifiers = 0u32;
    for part in parts.iter().take(parts.len() - 1) {
        let mod_ = parse_modifier(part)?;
        modifiers |= mod_;
    }
    let vk = key_name_to_vk(key_name)?;
    modifiers |= MOD_NOREPEAT;
    Some((modifiers, vk))
}
