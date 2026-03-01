using System.Runtime.InteropServices;

namespace Interop;

/// <summary>Public API for window operations (foreground, bounds, visibility).</summary>
public static class WindowInterop
{
    /// <summary>Gets the foreground window handle, or null if none.</summary>
    public static nint GetForegroundWindow()
    {
        var hwnd = User32.GetForegroundWindow();
        return User32.IsWindow(hwnd) ? hwnd : nint.Zero;
    }

    /// <summary>Gets the window bounds in screen coordinates. Prefers DWM extended frame bounds for accuracy.
    /// Set useWindowRect to true for Center so read/set match (avoids shrinking loop; DWM bounds can differ from SetWindowPos).</summary>
    public static bool TryGetWindowBounds(nint hwnd, out RECT rect, bool useWindowRect = false)
    {
        rect = default;
        if (hwnd == nint.Zero || !User32.IsWindow(hwnd)) return false;
        if (useWindowRect)
            return User32.GetWindowRect(hwnd, out rect);
        if (DwmApi.DwmGetWindowAttribute(hwnd, DwmApi.DWMWINDOWATTRIBUTE.DWMWA_EXTENDED_FRAME_BOUNDS, out rect, Marshal.SizeOf<RECT>()) == 0)
            return true;
        return User32.GetWindowRect(hwnd, out rect);
    }

    /// <summary>Sets the window position and size. Uses SWP_NOACTIVATE so focus is not stolen.</summary>
    public static bool SetWindowBounds(nint hwnd, RECT rect, bool activate = false)
    {
        if (hwnd == nint.Zero || !User32.IsWindow(hwnd)) return false;
        uint flags = User32.SWP_NOZORDER | (uint)(activate ? 0 : User32.SWP_NOACTIVATE);
        return User32.SetWindowPos(hwnd, User32.HWND_TOP, rect.Left, rect.Top, rect.Width, rect.Height, flags);
    }

    /// <summary>Returns true if the window is visible (and not cloaked if we check DWM).</summary>
    public static bool IsWindowVisible(nint hwnd) => hwnd != nint.Zero && User32.IsWindowVisible(hwnd);

    /// <summary>Enumerates top-level windows; callback receives each HWND. Return false from callback to stop.</summary>
    public static void EnumWindows(Func<nint, bool> callback)
    {
        if (callback == null) return;
        User32.EnumWindows((h, _) => callback(h), nint.Zero);
    }

    /// <summary>Brings the window to the foreground.</summary>
    public static bool SetForegroundWindow(nint hwnd) => hwnd != nint.Zero && User32.SetForegroundWindow(hwnd);

    /// <summary>Restores a minimized window.</summary>
    public static bool ShowWindow(nint hwnd, int nCmdShow = User32.SW_RESTORE) =>
        hwnd != nint.Zero && User32.ShowWindow(hwnd, nCmdShow) != 0;
}
