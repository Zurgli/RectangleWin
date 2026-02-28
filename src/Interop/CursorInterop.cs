namespace Interop;

/// <summary>Public API for cursor position (for "move cursor" option).</summary>
public static class CursorInterop
{
    /// <summary>Gets the cursor position in screen coordinates.</summary>
    public static bool TryGetCursorPos(out POINT pt)
    {
        pt = default;
        return User32.GetCursorPos(out pt);
    }

    /// <summary>Sets the cursor position in screen coordinates.</summary>
    public static bool SetCursorPos(int x, int y) => User32.SetCursorPos(x, y);

    /// <summary>Sets the cursor position to the center of the given rect.</summary>
    public static bool SetCursorPosToRect(RECT rect)
    {
        int x = rect.Left + rect.Width / 2;
        int y = rect.Top + rect.Height / 2;
        return User32.SetCursorPos(x, y);
    }
}
