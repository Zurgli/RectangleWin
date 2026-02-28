using Interop;
using WindowEngine;

namespace Core;

/// <summary>Options for Execute (from config).</summary>
public sealed class ExecuteOptions
{
    public bool UseCursorScreen { get; set; }
    public bool MoveCursorAfterSnap { get; set; }
    public bool MoveCursorAcrossDisplays { get; set; }
    public float GapSize { get; set; }
    public bool UpdateRestoreRect { get; set; } = true;
    public HashSet<string>? DisabledProcessNames { get; set; }
}

/// <summary>Orchestrates: resolve window -> work area -> calculation -> gaps -> SetWindowPos -> history.</summary>
public sealed class WindowManager
{
    private readonly MonitorManager.MonitorManager _monitorManager = new();
    private readonly WindowHistory _history = new();

    public WindowHistory History => _history;

    /// <summary>Execute a window action on the foreground window (or specified hwnd).</summary>
    public bool Execute(WindowAction action, nint? hwnd = null, ExecuteOptions? options = null)
    {
        options ??= new ExecuteOptions();
        nint target = hwnd ?? WindowInterop.GetForegroundWindow();
        if (target == nint.Zero) return false;

        if (options.DisabledProcessNames?.Count > 0)
        {
            string? name = ProcessInterop.GetProcessImageFileName(target);
            if (name != null && options.DisabledProcessNames.Contains(name, StringComparer.OrdinalIgnoreCase))
                return false;
        }

        if (action == WindowAction.Restore)
        {
            if (_history.GetRestoreRect(target) is not { } restore)
                return false;
            bool ok = WindowInterop.SetWindowBounds(target, restore, activate: false);
            if (ok) _history.RemoveLastAction(target);
            return ok;
        }

        if (action == WindowAction.NextDisplay || action == WindowAction.PreviousDisplay)
        {
            if (!WindowInterop.TryGetWindowBounds(target, out RECT currentRect))
                return false;
            var (workArea, prevWorkArea, nextWorkArea) = _monitorManager.GetCurrentAndAdjacentWorkAreas(target, options.UseCursorScreen);
            RECT? targetWorkArea = action == WindowAction.NextDisplay ? nextWorkArea : prevWorkArea;
            if (targetWorkArea is not { } dest)
                return false;
            if (options.UpdateRestoreRect)
                _history.SetRestoreRect(target, currentRect);
            var engineRect = dest.ToEngine();
            if (options.GapSize > 0)
                engineRect = GapCalculation.ApplyGaps(engineRect, Dimension.Both, Edge.None, options.GapSize);
            bool ok = WindowInterop.SetWindowBounds(target, engineRect.ToInterop(), activate: false);
            if (ok)
            {
                _history.SetLastAction(target, new RectangleAction(WindowAction.Maximize, engineRect.ToInterop()));
                if (options.MoveCursorAcrossDisplays)
                    CursorInterop.SetCursorPosToRect(engineRect.ToInterop());
                WindowInterop.SetForegroundWindow(target);
            }
            return ok;
        }

        IWindowCalculation? calculation = WindowCalculationFactory.GetCalculation(action);
        if (calculation == null) return false;

        if (!WindowInterop.TryGetWindowBounds(target, out RECT windowRect))
            return false;

        var (currentWorkArea, _, _) = _monitorManager.GetCurrentAndAdjacentWorkAreas(target, options.UseCursorScreen);
        RECT work = currentWorkArea;
        if (work.Left == 0 && work.Top == 0 && work.Right == 0 && work.Bottom == 0)
            return false;

        LastActionInfo? lastInfo = _history.GetLastAction(target) is { } la
            ? new LastActionInfo(la.Rect.ToEngine(), la.Action)
            : null;

        var parameters = new RectCalculationParameters(
            windowRect.ToEngine(),
            work.ToEngine(),
            action,
            lastInfo);

        CalculationResult? result = calculation.Calculate(parameters);
        if (result is not { } r) return false;

        Rect targetRect = r.Rect;
        if (options.GapSize > 0)
            targetRect = GapCalculation.ApplyGaps(targetRect, Dimension.Both, Edge.None, options.GapSize);

        if (options.UpdateRestoreRect)
            _history.SetRestoreRect(target, windowRect);

        bool applied = WindowInterop.SetWindowBounds(target, targetRect.ToInterop(), activate: false);
        if (applied)
        {
            _history.SetLastAction(target, new RectangleAction(r.ResultingAction, targetRect.ToInterop()));
            if (options.MoveCursorAfterSnap)
                CursorInterop.SetCursorPosToRect(targetRect.ToInterop());
        }
        return applied;
    }
}
