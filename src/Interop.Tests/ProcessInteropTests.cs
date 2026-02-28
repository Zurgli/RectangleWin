using Interop;

namespace Interop.Tests;

public sealed class ProcessInteropTests
{
    [Fact]
    public void GetProcessImageFileName_zero_returns_null()
    {
        Assert.Null(ProcessInterop.GetProcessImageFileName(nint.Zero));
    }

    [Fact]
    public void GetProcessImageFileName_foreground_returns_exe_name()
    {
        nint hwnd = WindowInterop.GetForegroundWindow();
        if (hwnd == nint.Zero) return;
        string? name = ProcessInterop.GetProcessImageFileName(hwnd);
        // May be null for protected processes; otherwise non-empty exe name
        if (name != null)
            Assert.False(string.IsNullOrWhiteSpace(name));
    }
}
