using Interop;

namespace MonitorManager;

/// <summary>Provides current and adjacent monitor work areas for the layout engine.</summary>
public sealed class MonitorManager
{
    /// <summary>Gets the work area for the monitor containing the given window, or null if invalid.</summary>
    public RECT? GetWorkAreaForWindow(nint hwnd)
    {
        if (hwnd == nint.Zero) return null;
        nint hMon = MonitorInterop.GetMonitorFromWindow(hwnd);
        if (hMon == nint.Zero) return null;
        if (!MonitorInterop.TryGetMonitorInfo(hMon, out _, out RECT workArea)) return null;
        return workArea;
    }

    /// <summary>Gets the work area for the monitor containing the cursor.</summary>
    public RECT? GetWorkAreaAtCursor()
    {
        if (!CursorInterop.TryGetCursorPos(out POINT pt)) return null;
        nint hMon = MonitorInterop.GetMonitorFromPoint(pt);
        if (hMon == nint.Zero) return null;
        if (!MonitorInterop.TryGetMonitorInfo(hMon, out _, out RECT workArea)) return null;
        return workArea;
    }

    /// <summary>Gets current work area: by window or by cursor depending on useCursorScreen.</summary>
    public RECT? GetCurrentWorkArea(nint? foregroundWindow, bool useCursorScreen)
    {
        if (useCursorScreen)
            return GetWorkAreaAtCursor();
        if (foregroundWindow is { } hwnd and not 0)
            return GetWorkAreaForWindow(hwnd);
        return GetWorkAreaAtCursor();
    }

    /// <summary>Ordered list of monitors (left-to-right, top-to-bottom).</summary>
    public IReadOnlyList<MonitorInfo> GetOrderedMonitors() => MonitorInterop.EnumMonitors();

    /// <summary>Gets the adjacent monitor's work area (previous or next in physical order). Returns null if only one monitor.</summary>
    public RECT? GetAdjacentWorkArea(nint currentHMonitor, bool next)
    {
        var list = GetOrderedMonitors();
        if (list.Count <= 1) return null;
        int idx = -1;
        for (int i = 0; i < list.Count; i++)
        {
            if (list[i].HMonitor == currentHMonitor) { idx = i; break; }
        }
        if (idx < 0) return null;
        int adj = next ? idx + 1 : idx - 1;
        if (adj < 0 || adj >= list.Count) return null;
        return list[adj].WorkArea;
    }

    /// <summary>Gets current monitor handle and work area; and the adjacent work area for next/prev display.</summary>
    public (RECT workArea, RECT? previousWorkArea, RECT? nextWorkArea) GetCurrentAndAdjacentWorkAreas(nint? foregroundWindow, bool useCursorScreen)
    {
        RECT? current = GetCurrentWorkArea(foregroundWindow, useCursorScreen);
        if (current is not { } workArea)
            return (default, null, null);

        nint hMon = foregroundWindow.HasValue && foregroundWindow.Value != nint.Zero && !useCursorScreen
            ? MonitorInterop.GetMonitorFromWindow(foregroundWindow.Value)
            : (CursorInterop.TryGetCursorPos(out POINT pt) ? MonitorInterop.GetMonitorFromPoint(pt) : nint.Zero);
        if (hMon == nint.Zero)
            return (workArea, null, null);

        RECT? prev = GetAdjacentWorkArea(hMon, next: false);
        RECT? next = GetAdjacentWorkArea(hMon, next: true);
        return (workArea, prev, next);
    }
}
