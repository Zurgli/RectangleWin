using System.Text.Json;
using System.Text.Json.Serialization;

namespace TrayApp;

public sealed class AppConfig
{
    public float GapSize { get; set; }
    public bool UseCursorScreen { get; set; }
    public bool MoveCursorAfterSnap { get; set; }
    public bool MoveCursorAcrossDisplays { get; set; }
    public bool DebugLogging { get; set; }
    public List<HotkeyBinding> Hotkeys { get; set; } = new();

    public static AppConfig Default()
    {
        uint winAlt = Interop.HotkeyWin32.MOD_WIN | Interop.HotkeyWin32.MOD_ALT | Interop.HotkeyWin32.MOD_NOREPEAT;
        return new AppConfig
        {
            GapSize = 0,
            Hotkeys = new List<HotkeyBinding>
            {
                new("LeftHalf", winAlt, 0x31),      // 1
                new("RightHalf", winAlt, 0x32),     // 2
                new("TopHalf", winAlt, 0x33),       // 3
                new("BottomHalf", winAlt, 0x34),    // 4
                new("Maximize", winAlt, 0x35),      // 5
                new("Center", winAlt, 0x36),       // 6
                new("UpperLeft", winAlt, 0x51),     // Q
                new("UpperRight", winAlt, 0x57),   // W
                new("LowerLeft", winAlt, 0x41),    // A
                new("LowerRight", winAlt, 0x53),   // S
                new("NextDisplay", winAlt, 0x4E),  // N
                new("PreviousDisplay", winAlt, 0x50), // P
                new("Restore", winAlt, 0x52),      // R
            }
        };
    }

    public static string ConfigPath()
    {
        string dir = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "RectangleWin");
        Directory.CreateDirectory(dir);
        return Path.Combine(dir, "config.json");
    }

    public static AppConfig Load()
    {
        string path = ConfigPath();
        if (!File.Exists(path))
            return Default();
        try
        {
            string json = File.ReadAllText(path);
            var c = JsonSerializer.Deserialize<AppConfig>(json);
            return c ?? Default();
        }
        catch
        {
            return Default();
        }
    }

    public void Save()
    {
        string path = ConfigPath();
        var opts = new JsonSerializerOptions { WriteIndented = true };
        File.WriteAllText(path, JsonSerializer.Serialize(this, opts));
    }
}

public record HotkeyBinding(
    [property: JsonPropertyName("action")] string Action,
    [property: JsonPropertyName("modifiers")] uint Modifiers,
    [property: JsonPropertyName("virtualKey")] uint VirtualKey);
