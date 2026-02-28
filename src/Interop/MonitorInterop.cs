namespace Interop;

/// <summary>Monitor handle and work area; for use by MonitorManager.</summary>
public readonly record struct MonitorInfo(nint HMonitor, RECT MonitorRect, RECT WorkArea);

/// <summary>Public API for monitor enumeration and work area.</summary>
public static class MonitorInterop
{
    /// <summary>Gets the monitor containing the given window (nearest if overlapping).</summary>
    public static nint GetMonitorFromWindow(nint hwnd) =>
        User32.MonitorFromWindow(hwnd, User32.MONITOR_DEFAULTTONEAREST);

    /// <summary>Gets the monitor containing the given point (cursor or other).</summary>
    public static nint GetMonitorFromPoint(POINT pt) =>
        User32.MonitorFromPoint(pt, User32.MONITOR_DEFAULTTONEAREST);

    /// <summary>Gets monitor and work area for the given HMONITOR.</summary>
    public static bool TryGetMonitorInfo(nint hMonitor, out RECT monitorRect, out RECT workArea)
    {
        monitorRect = default;
        workArea = default;
        if (hMonitor == nint.Zero) return false;
        var mi = MONITORINFO.Create();
        if (!User32.GetMonitorInfo(hMonitor, ref mi)) return false;
        monitorRect = mi.rcMonitor;
        workArea = mi.rcWork;
        return true;
    }

    /// <summary>Enumerates all display monitors in physical order (left-to-right, top-to-bottom).</summary>
    public static IReadOnlyList<MonitorInfo> EnumMonitors()
    {
        var list = new List<MonitorInfo>();
        User32.EnumDisplayMonitors(nint.Zero, nint.Zero, EnumProc, nint.Zero);
        list.Sort((a, b) =>
        {
            int c = a.MonitorRect.Left.CompareTo(b.MonitorRect.Left);
            if (c != 0) return c;
            return a.MonitorRect.Top.CompareTo(b.MonitorRect.Top);
        });
        return list;

        bool EnumProc(nint hMonitor, nint hdcMonitor, ref RECT lprcMonitor, nint dwData)
        {
            var mi = MONITORINFO.Create();
            if (User32.GetMonitorInfo(hMonitor, ref mi))
                list.Add(new MonitorInfo(hMonitor, mi.rcMonitor, mi.rcWork));
            return true;
        }
    }
}
