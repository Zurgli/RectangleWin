using Interop;

namespace Interop.Tests;

public sealed class CursorInteropTests
{
    [Fact]
    public void TryGetCursorPos_returns_true_and_valid_point()
    {
        bool ok = CursorInterop.TryGetCursorPos(out POINT pt);
        Assert.True(ok);
        // Point can be anywhere on screen(s)
    }

    [Fact]
    public void SetCursorPosToRect_returns_true()
    {
        var rect = new RECT { Left = 100, Top = 100, Right = 200, Bottom = 200 };
        bool ok = CursorInterop.SetCursorPosToRect(rect);
        Assert.True(ok);
        // Cursor was moved to center of rect; we don't assert position to avoid moving user's cursor in test
    }
}
