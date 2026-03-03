using System.Text.Json;
using System.Text.Json.Serialization;

namespace TrayApp;

public sealed class AppConfig
{
    // --- General ---
    [JsonPropertyName("launchOnLogin")]
    public bool LaunchOnLogin { get; set; }
    [JsonPropertyName("disabledApps")]
    public List<string> DisabledApps { get; set; } = new();
    [JsonPropertyName("hideMenuBarIcon")]
    public bool HideMenuBarIcon { get; set; }
    [JsonPropertyName("debugLogging")]
    public bool DebugLogging { get; set; }
    [JsonPropertyName("allowAnyShortcut")]
    public bool AllowAnyShortcut { get; set; }

    // --- Repeated command / cycling (Rectangle: subsequentExecutionMode) ---
    /// <summary>0=halves to thirds, 1=cycle displays, 2=disabled, 3=mixed, 4=repeat on next display</summary>
    [JsonPropertyName("subsequentExecutionMode")]
    public int SubsequentExecutionMode { get; set; }
    [JsonPropertyName("traverseSingleScreen")]
    public bool TraverseSingleScreen { get; set; }
    [JsonPropertyName("attemptMatchOnNextPrevDisplay")]
    public bool AttemptMatchOnNextPrevDisplay { get; set; }

    // --- Screen / cursor (accept old names so existing config still applies) ---
    [JsonPropertyName("UseCursorScreen")]
    public bool UseCursorScreenDetection { get; set; }
    [JsonPropertyName("MoveCursorAfterSnap")]
    public bool MoveCursor { get; set; }
    [JsonPropertyName("MoveCursorAcrossDisplays")]
    public bool MoveCursorAcrossDisplays { get; set; }
    [JsonPropertyName("screensOrderedByX")]
    public bool ScreensOrderedByX { get; set; } = true;

    // --- Gaps ---
    /// <summary>Gap between windows (positive). Negative = overdraw to compensate for semi-transparent window edges.</summary>
    [JsonPropertyName("gapSize")]
    public float GapSize { get; set; }
    [JsonPropertyName("screenEdgeGapTop")]
    public float ScreenEdgeGapTop { get; set; }
    [JsonPropertyName("screenEdgeGapBottom")]
    public float ScreenEdgeGapBottom { get; set; }
    [JsonPropertyName("screenEdgeGapLeft")]
    public float ScreenEdgeGapLeft { get; set; }
    [JsonPropertyName("screenEdgeGapRight")]
    public float ScreenEdgeGapRight { get; set; }
    [JsonPropertyName("screenEdgeGapsOnMainScreenOnly")]
    public bool ScreenEdgeGapsOnMainScreenOnly { get; set; }
    /// <summary>Pixels to extend work area bottom (fixes Windows 11 gap above taskbar; try 10). 0 = off.</summary>
    [JsonPropertyName("taskbarGapCompensation")]
    public int TaskbarGapCompensation { get; set; }
    /// <summary>Pixels to extend work area left (fixes gap at left edge). 0 = off.</summary>
    [JsonPropertyName("taskbarGapCompensationLeft")]
    public int TaskbarGapCompensationLeft { get; set; }
    /// <summary>Pixels to extend work area right (fixes gap at right edge). 0 = off.</summary>
    [JsonPropertyName("taskbarGapCompensationRight")]
    public int TaskbarGapCompensationRight { get; set; }
    [JsonPropertyName("applyGapsToMaximize")]
    public bool ApplyGapsToMaximize { get; set; } = true;
    [JsonPropertyName("applyGapsToMaximizeHeight")]
    public bool ApplyGapsToMaximizeHeight { get; set; } = true;

    // --- Move / resize behavior ---
    [JsonPropertyName("resizeOnDirectionalMove")]
    public bool ResizeOnDirectionalMove { get; set; }
    [JsonPropertyName("centeredDirectionalMove")]
    public bool? CenteredDirectionalMove { get; set; }
    [JsonPropertyName("unsnapRestore")]
    public bool? UnsnapRestore { get; set; }

    // --- Sizing limits / almost maximize ---
    [JsonPropertyName("minimumWindowWidth")]
    public float MinimumWindowWidth { get; set; }
    [JsonPropertyName("minimumWindowHeight")]
    public float MinimumWindowHeight { get; set; }
    [JsonPropertyName("almostMaximizeHeight")]
    public float AlmostMaximizeHeight { get; set; }
    [JsonPropertyName("almostMaximizeWidth")]
    public float AlmostMaximizeWidth { get; set; }
    [JsonPropertyName("sizeOffset")]
    public float SizeOffset { get; set; }
    [JsonPropertyName("widthStepSize")]
    public float WidthStepSize { get; set; } = 30f;

    // --- Custom / specified ---
    [JsonPropertyName("specifiedWidth")]
    public float SpecifiedWidth { get; set; } = 1680f;
    [JsonPropertyName("specifiedHeight")]
    public float SpecifiedHeight { get; set; } = 1050f;
    [JsonPropertyName("horizontalSplitRatio")]
    public float HorizontalSplitRatio { get; set; } = 50f;
    [JsonPropertyName("verticalSplitRatio")]
    public float VerticalSplitRatio { get; set; } = 50f;

    // --- Hotkeys (Win+Alt by default) ---
    [JsonPropertyName("hotkeys")]
    public List<HotkeyBinding> Hotkeys { get; set; } = new();

    // Back-compat: old config may have PascalCase or different names
    [JsonIgnore]
    public bool UseCursorScreen { get => UseCursorScreenDetection; set => UseCursorScreenDetection = value; }
    [JsonIgnore]
    public bool MoveCursorAfterSnap { get => MoveCursor; set => MoveCursor = value; }

    public static AppConfig Default()
    {
        uint winAlt = Interop.HotkeyWin32.MOD_WIN | Interop.HotkeyWin32.MOD_ALT | Interop.HotkeyWin32.MOD_NOREPEAT;
        return new AppConfig
        {
            LaunchOnLogin = true,
            GapSize = -2f,
            ScreenEdgeGapTop = 0,
            ScreenEdgeGapBottom = 0,
            ScreenEdgeGapLeft = 0,
            ScreenEdgeGapRight = 0,
            TaskbarGapCompensation = 0,
            TaskbarGapCompensationLeft = 0,
            TaskbarGapCompensationRight = 0,
            ApplyGapsToMaximize = true,
            ApplyGapsToMaximizeHeight = true,
            SpecifiedWidth = 1680f,
            SpecifiedHeight = 1050f,
            HorizontalSplitRatio = 50f,
            VerticalSplitRatio = 50f,
            WidthStepSize = 30f,
            Hotkeys = new List<HotkeyBinding>
            {
                new("LeftHalf", winAlt, 0x25),      // Left
                new("RightHalf", winAlt, 0x27),    // Right
                new("TopHalf", winAlt, 0x26),      // Up
                new("BottomHalf", winAlt, 0x28),   // Down
                new("UpperLeft", winAlt, 0x55),     // U
                new("UpperRight", winAlt, 0x49),   // I
                new("LowerLeft", winAlt, 0x4A),    // J
                new("LowerRight", winAlt, 0x4B),   // K
                new("Maximize", winAlt, 0x0D),     // Enter
                new("Center", winAlt, 0x43),       // C
                new("Undo", winAlt, 0x2E),      // Delete
                new("FirstThird", winAlt, 0x44),   // D - left 1/3
                new("FirstTwoThirds", winAlt, 0x45), // E - left 2/3
                new("CenterThird", winAlt, 0x46), // F - center 1/3
                new("LastTwoThirds", winAlt, 0x54), // T - right 2/3
                new("LastThird", winAlt, 0x47),   // G - right 1/3
                new("CenterTwoThirds", winAlt, 0x52), // R - center 2/3
                new("NextDisplay", winAlt, 0x4E),  // N
                new("PreviousDisplay", winAlt, 0x50), // P
            }
        };
    }

    public static string ConfigPath()
    {
        string dir = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "RectangleWin");
        Directory.CreateDirectory(dir);
        return Path.Combine(dir, "config.json");
    }

    /// <summary>Only these properties are read from / written to config.json. Everything else uses defaults.</summary>
    private sealed class PersistedConfig
    {
        [JsonPropertyName("launchOnLogin")]
        public bool LaunchOnLogin { get; set; }
        [JsonPropertyName("gapSize")]
        public float GapSize { get; set; }
        [JsonPropertyName("screenEdgeGapTop")]
        public float ScreenEdgeGapTop { get; set; }
        [JsonPropertyName("screenEdgeGapBottom")]
        public float ScreenEdgeGapBottom { get; set; }
        [JsonPropertyName("screenEdgeGapLeft")]
        public float ScreenEdgeGapLeft { get; set; }
        [JsonPropertyName("screenEdgeGapRight")]
        public float ScreenEdgeGapRight { get; set; }
        [JsonPropertyName("screenEdgeGapsOnMainScreenOnly")]
        public bool ScreenEdgeGapsOnMainScreenOnly { get; set; }
        [JsonPropertyName("taskbarGapCompensation")]
        public int TaskbarGapCompensation { get; set; }
        [JsonPropertyName("taskbarGapCompensationLeft")]
        public int TaskbarGapCompensationLeft { get; set; }
        [JsonPropertyName("taskbarGapCompensationRight")]
        public int TaskbarGapCompensationRight { get; set; }
        [JsonPropertyName("applyGapsToMaximize")]
        public bool ApplyGapsToMaximize { get; set; } = true;
        [JsonPropertyName("applyGapsToMaximizeHeight")]
        public bool ApplyGapsToMaximizeHeight { get; set; } = true;
        [JsonPropertyName("hotkeys")]
        public List<HotkeyBinding> Hotkeys { get; set; } = new();
    }

    public static AppConfig Load()
    {
        string path = ConfigPath();
        var result = Default();
        if (!File.Exists(path))
            return result;
        try
        {
            string json = File.ReadAllText(path);
            var opts = new JsonSerializerOptions { PropertyNameCaseInsensitive = true };
            var p = JsonSerializer.Deserialize<PersistedConfig>(json, opts);
            if (p == null) return result;
            result.LaunchOnLogin = p.LaunchOnLogin;
            result.GapSize = p.GapSize;
            result.ScreenEdgeGapTop = p.ScreenEdgeGapTop;
            result.ScreenEdgeGapBottom = p.ScreenEdgeGapBottom;
            result.ScreenEdgeGapLeft = p.ScreenEdgeGapLeft;
            result.ScreenEdgeGapRight = p.ScreenEdgeGapRight;
            result.ScreenEdgeGapsOnMainScreenOnly = p.ScreenEdgeGapsOnMainScreenOnly;
            result.TaskbarGapCompensation = p.TaskbarGapCompensation;
            result.TaskbarGapCompensationLeft = p.TaskbarGapCompensationLeft;
            result.TaskbarGapCompensationRight = p.TaskbarGapCompensationRight;
            result.ApplyGapsToMaximize = p.ApplyGapsToMaximize;
            result.ApplyGapsToMaximizeHeight = p.ApplyGapsToMaximizeHeight;
            if (p.Hotkeys?.Count > 0)
                result.Hotkeys = p.Hotkeys;
            return result;
        }
        catch
        {
            return result;
        }
    }

    public void Save()
    {
        string path = ConfigPath();
        var p = new PersistedConfig
        {
            LaunchOnLogin = LaunchOnLogin,
            GapSize = GapSize,
            ScreenEdgeGapTop = ScreenEdgeGapTop,
            ScreenEdgeGapBottom = ScreenEdgeGapBottom,
            ScreenEdgeGapLeft = ScreenEdgeGapLeft,
            ScreenEdgeGapRight = ScreenEdgeGapRight,
            ScreenEdgeGapsOnMainScreenOnly = ScreenEdgeGapsOnMainScreenOnly,
            TaskbarGapCompensation = TaskbarGapCompensation,
            TaskbarGapCompensationLeft = TaskbarGapCompensationLeft,
            TaskbarGapCompensationRight = TaskbarGapCompensationRight,
            ApplyGapsToMaximize = ApplyGapsToMaximize,
            ApplyGapsToMaximizeHeight = ApplyGapsToMaximizeHeight,
            Hotkeys = Hotkeys
        };
        var opts = new JsonSerializerOptions { WriteIndented = true };
        File.WriteAllText(path, JsonSerializer.Serialize(p, opts));
    }
}

public record HotkeyBinding(
    [property: JsonPropertyName("action")] string Action,
    [property: JsonPropertyName("modifiers")] uint Modifiers,
    [property: JsonPropertyName("virtualKey")] uint VirtualKey);
