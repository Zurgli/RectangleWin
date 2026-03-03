using System.Collections.Generic;
using System.Linq;

namespace TrayApp;

/// <summary>Format and parse human-readable shortcuts (e.g. "Win+Alt+Left") for config and UI.</summary>
public static class ShortcutHelper
{
    private const uint MOD_NOREPEAT = Interop.HotkeyWin32.MOD_NOREPEAT;

    public static string FormatShortcut(uint modifiers, uint vk)
    {
        var parts = new List<string>();
        if ((modifiers & Interop.HotkeyWin32.MOD_WIN) != 0) parts.Add("Win");
        if ((modifiers & Interop.HotkeyWin32.MOD_ALT) != 0) parts.Add("Alt");
        if ((modifiers & Interop.HotkeyWin32.MOD_CONTROL) != 0) parts.Add("Ctrl");
        if ((modifiers & Interop.HotkeyWin32.MOD_SHIFT) != 0) parts.Add("Shift");
        parts.Add(VkToKeyName(vk));
        return string.Join("+", parts);
    }

    /// <summary>Parse a shortcut string like "Win+Alt+Left" into modifiers and virtual key. Adds MOD_NOREPEAT to modifiers. Returns false if parsing fails.</summary>
    public static bool TryParseShortcut(string shortcut, out uint modifiers, out uint virtualKey)
    {
        modifiers = 0;
        virtualKey = 0;
        if (string.IsNullOrWhiteSpace(shortcut)) return false;
        var parts = shortcut.Split('+').Select(p => p.Trim()).Where(p => p.Length > 0).ToList();
        if (parts.Count == 0) return false;
        string keyName = parts[^1];
        for (int i = 0; i < parts.Count - 1; i++)
        {
            uint mod = parts[i].ToLowerInvariant() switch
            {
                "win" => Interop.HotkeyWin32.MOD_WIN,
                "alt" => Interop.HotkeyWin32.MOD_ALT,
                "ctrl" or "control" => Interop.HotkeyWin32.MOD_CONTROL,
                "shift" => Interop.HotkeyWin32.MOD_SHIFT,
                _ => 0
            };
            if (mod == 0) return false;
            modifiers |= mod;
        }
        if (!KeyNameToVk(keyName, out virtualKey)) return false;
        modifiers |= MOD_NOREPEAT;
        return true;
    }

    public static string VkToKeyName(uint vk)
    {
        return vk switch
        {
            0x25 => "Left",
            0x26 => "Up",
            0x27 => "Right",
            0x28 => "Down",
            0x0D => "Enter",
            0x2E => "Delete",
            0x31 => "1", 0x32 => "2", 0x33 => "3", 0x34 => "4", 0x35 => "5", 0x36 => "6",
            0x41 => "A", 0x42 => "B", 0x43 => "C", 0x44 => "D", 0x45 => "E", 0x46 => "F",
            0x47 => "G", 0x48 => "H", 0x49 => "I", 0x4A => "J", 0x4B => "K", 0x4E => "N",
            0x50 => "P", 0x51 => "Q", 0x52 => "R", 0x53 => "S", 0x54 => "T", 0x55 => "U",
            0x57 => "W",
            _ => $"0x{vk:X}"
        };
    }

    private static bool KeyNameToVk(string name, out uint vk)
    {
        vk = 0;
        if (string.IsNullOrEmpty(name)) return false;
        var n = name.Trim();
        if (n.Length == 1)
        {
            char c = n[0];
            if (c >= '0' && c <= '9') { vk = (uint)(0x31 + (c - '0')); return true; }
            if (c >= 'A' && c <= 'Z') { vk = (uint)(0x41 + (c - 'A')); return true; }
            if (c >= 'a' && c <= 'z') { vk = (uint)(0x41 + (c - 'a')); return true; }
        }
        vk = n.ToLowerInvariant() switch
        {
            "left" => 0x25,
            "up" => 0x26,
            "right" => 0x27,
            "down" => 0x28,
            "enter" => 0x0D,
            "delete" => 0x2E,
            "1" => 0x31, "2" => 0x32, "3" => 0x33, "4" => 0x34, "5" => 0x35, "6" => 0x36,
            "a" => 0x41, "b" => 0x42, "c" => 0x43, "d" => 0x44, "e" => 0x45, "f" => 0x46,
            "g" => 0x47, "h" => 0x48, "i" => 0x49, "j" => 0x4A, "k" => 0x4B, "n" => 0x4E,
            "p" => 0x50, "q" => 0x51, "r" => 0x52, "s" => 0x53, "t" => 0x54, "u" => 0x55,
            "w" => 0x57,
            _ => 0
        };
        return vk != 0;
    }
}
