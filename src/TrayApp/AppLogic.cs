using Core;
using Microsoft.UI.Dispatching;
using WindowEngine;

namespace TrayApp;

/// <summary>Holds WindowManager, HotkeyManager, config, and wires hotkeys to Execute.</summary>
public sealed class AppLogic
{
    private readonly HotkeyManager.HotkeyManager _hotkeyManager = new();
    private readonly Dictionary<int, WindowAction> _idToAction = new();
    private readonly DispatcherQueue _dispatcher;
    private WindowManager? _windowManager;

    public AppConfig Config { get; private set; }
    public WindowManager WindowManager => _windowManager ?? throw new InvalidOperationException("Not started");
    public HotkeyManager.HotkeyManager HotkeyManager => _hotkeyManager;

    /// <summary>Raised when a hotkey is pressed (on UI thread), with the action name. For UI feedback.</summary>
    public event Action<string>? HotkeyTriggered;

    public AppLogic(DispatcherQueue dispatcher)
    {
        _dispatcher = dispatcher;
        Config = AppConfig.Load();
    }

    public void Start()
    {
        _windowManager = new WindowManager();

        _hotkeyManager.HotkeyPressed += OnHotkeyPressed;
        AppLog.DebugEnabled = Config.DebugLogging;
        if (Config.DebugLogging)
            _hotkeyManager.SetDiagnosticLog(AppLog.Write);
        _hotkeyManager.Start();

        int registered = 0;
        foreach (var binding in Config.Hotkeys)
        {
            if (!TryParseWindowAction(binding.Action, out var action))
                continue;
            var result = _hotkeyManager.Register(binding.Modifiers, binding.VirtualKey);
            if (result.Id is { } i)
            {
                _idToAction[i] = action;
                registered++;
            }
            else if (result.ErrorCode is { } err)
                AppLog.Write($"Hotkey failed: {binding.Action} -> Win32 error {err} (e.g. 1409 = already registered)");
        }
        AppLog.WriteDebug($"Registered {registered}/{Config.Hotkeys.Count} hotkeys");
    }

    public void Stop()
    {
        _hotkeyManager.HotkeyPressed -= OnHotkeyPressed;
        _hotkeyManager.Stop();
        _idToAction.Clear();
    }

    public void SaveConfig()
    {
        Config.Save();
    }

    private static bool TryParseWindowAction(string? actionName, out WindowAction action)
    {
        if (Enum.TryParse<WindowAction>(actionName, ignoreCase: true, out action))
            return true;
        if (string.Equals(actionName, "Restore", StringComparison.OrdinalIgnoreCase))
        {
            action = WindowAction.Undo;
            return true;
        }
        return false;
    }

    /// <summary>Build options from current Config so hotkeys always use latest settings (e.g. gap size).</summary>
    private ExecuteOptions GetOptions()
    {
        return new ExecuteOptions
        {
            GapSize = Config.GapSize,
            UseCursorScreen = Config.UseCursorScreenDetection,
            MoveCursorAfterSnap = Config.MoveCursor,
            MoveCursorAcrossDisplays = Config.MoveCursorAcrossDisplays,
            DisabledProcessNames = Config.DisabledApps?.Count > 0 ? new HashSet<string>(Config.DisabledApps, StringComparer.OrdinalIgnoreCase) : null,
            ScreenEdgeGapTop = Config.ScreenEdgeGapTop,
            ScreenEdgeGapBottom = Config.ScreenEdgeGapBottom,
            ScreenEdgeGapLeft = Config.ScreenEdgeGapLeft,
            ScreenEdgeGapRight = Config.ScreenEdgeGapRight,
            ScreenEdgeGapsOnMainScreenOnly = Config.ScreenEdgeGapsOnMainScreenOnly,
            TaskbarGapCompensation = Config.TaskbarGapCompensation,
            TaskbarGapCompensationLeft = Config.TaskbarGapCompensationLeft,
            TaskbarGapCompensationRight = Config.TaskbarGapCompensationRight,
            ApplyGapsToMaximize = Config.ApplyGapsToMaximize,
            ApplyGapsToMaximizeHeight = Config.ApplyGapsToMaximizeHeight,
            SubsequentExecutionMode = Config.SubsequentExecutionMode,
            TraverseSingleScreen = Config.TraverseSingleScreen,
            SpecifiedWidth = Config.SpecifiedWidth,
            SpecifiedHeight = Config.SpecifiedHeight,
            AlmostMaximizeWidth = Config.AlmostMaximizeWidth,
            AlmostMaximizeHeight = Config.AlmostMaximizeHeight
        };
    }

    private void OnHotkeyPressed(int id)
    {
        if (!_idToAction.TryGetValue(id, out var action))
            return;
        string actionName = action.ToString();
        _dispatcher.TryEnqueue(() =>
        {
            HotkeyTriggered?.Invoke(actionName);
            _windowManager?.Execute(action, options: GetOptions());
        });
    }
}
