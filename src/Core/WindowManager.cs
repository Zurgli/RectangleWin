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
    // Screen edge gaps (inset work area)
    public float ScreenEdgeGapTop { get; set; }
    public float ScreenEdgeGapBottom { get; set; }
    public float ScreenEdgeGapLeft { get; set; }
    public float ScreenEdgeGapRight { get; set; }
    public bool ScreenEdgeGapsOnMainScreenOnly { get; set; }
    /// <summary>Pixels to add to work area bottom (Windows 11 taskbar gap fix).</summary>
    public int TaskbarGapCompensation { get; set; }
    public int TaskbarGapCompensationLeft { get; set; }
    public int TaskbarGapCompensationRight { get; set; }
    // Gaps for maximize actions
    public bool ApplyGapsToMaximize { get; set; } = true;
    public bool ApplyGapsToMaximizeHeight { get; set; } = true;
    // For future: subsequentExecutionMode, traverseSingleScreen, etc.
    public int SubsequentExecutionMode { get; set; }
    public bool TraverseSingleScreen { get; set; }
    public float SpecifiedWidth { get; set; } = 1680f;
    public float SpecifiedHeight { get; set; } = 1050f;
    public float AlmostMaximizeWidth { get; set; }
    public float AlmostMaximizeHeight { get; set; }
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
            dest = InsetWorkAreaByScreenEdgeGaps(dest, options);
            dest = ExtendWorkAreaBottomForTaskbarGap(dest, options);
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

        bool useWindowRectForBounds = (action == WindowAction.Center);
        if (!WindowInterop.TryGetWindowBounds(target, out RECT windowRect, useWindowRectForBounds))
            return false;

        var (currentWorkArea, _, _) = _monitorManager.GetCurrentAndAdjacentWorkAreas(target, options.UseCursorScreen);
        RECT work = currentWorkArea;
        if (work.Left == 0 && work.Top == 0 && work.Right == 0 && work.Bottom == 0)
            return false;
        work = InsetWorkAreaByScreenEdgeGaps(work, options);
        work = ExtendWorkAreaBottomForTaskbarGap(work, options);

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
        bool applyGaps = options.GapSize > 0;
        if (applyGaps && action == WindowAction.Maximize && !options.ApplyGapsToMaximize)
            applyGaps = false;
        if (applyGaps && action == WindowAction.Center)
            applyGaps = false; // Center only repositions; gaps would shrink the window on every press
        if (applyGaps)
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

    private static RECT InsetWorkAreaByScreenEdgeGaps(RECT work, ExecuteOptions options)
    {
        int t = (int)options.ScreenEdgeGapTop;
        int b = (int)options.ScreenEdgeGapBottom;
        int l = (int)options.ScreenEdgeGapLeft;
        int r = (int)options.ScreenEdgeGapRight;
        if (t == 0 && b == 0 && l == 0 && r == 0) return work;
        return new RECT
        {
            Left = work.Left + l,
            Top = work.Top + t,
            Right = work.Right - r,
            Bottom = work.Bottom - b
        };
    }

    /// <summary>Extend work area to fix Windows 11 gaps (rcWork leaves margin at bottom/left/right).</summary>
    private static RECT ExtendWorkAreaBottomForTaskbarGap(RECT work, ExecuteOptions options)
    {
        int bottom = options.TaskbarGapCompensation;
        int left = options.TaskbarGapCompensationLeft;
        int right = options.TaskbarGapCompensationRight;
        if (bottom <= 0 && left <= 0 && right <= 0) return work;
        return new RECT
        {
            Left = work.Left - left,
            Top = work.Top,
            Right = work.Right + right,
            Bottom = work.Bottom + bottom
        };
    }
}
