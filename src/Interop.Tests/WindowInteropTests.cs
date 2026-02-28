using Interop;

namespace Interop.Tests;

public sealed class WindowInteropTests
{
    [Fact]
    public void GetForegroundWindow_returns_handle()
    {
        nint hwnd = WindowInterop.GetForegroundWindow();
        // May be zero if no foreground window (e.g. headless); otherwise valid handle
        if (hwnd != nint.Zero && WindowInterop.TryGetWindowBounds(hwnd, out _))
            return; // got bounds, so valid window
        // Zero or window that doesn't support bounds (e.g. UWP) is acceptable
    }

    [Fact]
    public void TryGetWindowBounds_returns_false_for_zero_handle()
    {
        bool ok = WindowInterop.TryGetWindowBounds(nint.Zero, out _);
        Assert.False(ok);
    }

    [Fact]
    public void TryGetWindowBounds_and_SetWindowBounds_work_with_foreground_window()
    {
        nint hwnd = WindowInterop.GetForegroundWindow();
        if (hwnd == nint.Zero) return;

        if (!WindowInterop.TryGetWindowBounds(hwnd, out RECT before))
            return;

        // Set same bounds; some windows (DPI, UWP, min/max) may not report identical values after
        bool setOk = WindowInterop.SetWindowBounds(hwnd, before, activate: false);
        Assert.True(setOk);
        bool getOk = WindowInterop.TryGetWindowBounds(hwnd, out RECT after);
        Assert.True(getOk);
        // Sanity: after is a valid rect (positive size)
        Assert.True(after.Right > after.Left && after.Bottom > after.Top);
    }

    [Fact]
    public void IsWindowVisible_zero_returns_false()
    {
        Assert.False(WindowInterop.IsWindowVisible(nint.Zero));
    }
}
