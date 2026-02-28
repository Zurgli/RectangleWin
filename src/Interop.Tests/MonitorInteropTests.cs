using Interop;

namespace Interop.Tests;

public sealed class MonitorInteropTests
{
    [Fact]
    public void EnumMonitors_returns_at_least_one()
    {
        var monitors = MonitorInterop.EnumMonitors();
        Assert.NotEmpty(monitors);
    }

    [Fact]
    public void EnumMonitors_work_area_inside_monitor_rect()
    {
        var monitors = MonitorInterop.EnumMonitors();
        foreach (var m in monitors)
        {
            Assert.True(m.WorkArea.Left >= m.MonitorRect.Left);
            Assert.True(m.WorkArea.Top >= m.MonitorRect.Top);
            Assert.True(m.WorkArea.Right <= m.MonitorRect.Right);
            Assert.True(m.WorkArea.Bottom <= m.MonitorRect.Bottom);
        }
    }

    [Fact]
    public void GetMonitorFromPoint_returns_handle_for_cursor()
    {
        if (!CursorInterop.TryGetCursorPos(out POINT pt))
            return;
        nint hMon = MonitorInterop.GetMonitorFromPoint(pt);
        Assert.NotEqual(nint.Zero, hMon);
        Assert.True(MonitorInterop.TryGetMonitorInfo(hMon, out _, out _));
    }

    [Fact]
    public void GetMonitorFromWindow_foreground_returns_handle()
    {
        nint hwnd = WindowInterop.GetForegroundWindow();
        if (hwnd == nint.Zero) return;
        nint hMon = MonitorInterop.GetMonitorFromWindow(hwnd);
        Assert.NotEqual(nint.Zero, hMon);
    }
}
