using System.Runtime.InteropServices;

namespace Interop;

internal static partial class User32
{
    private const string DllName = "user32.dll";

    public const int SWP_NOACTIVATE = 0x0010;
    public const int SWP_NOZORDER = 0x0004;
    public const int SWP_NOMOVE = 0x0002;
    public const int SWP_NOSIZE = 0x0001;
    public static readonly nint HWND_TOP = nint.Zero;

    public const int MONITOR_DEFAULTTONEAREST = 2;
    public const int MONITOR_DEFAULTTOPRIMARY = 1;

    [LibraryImport(DllName)]
    public static partial nint GetForegroundWindow();

    [LibraryImport(DllName)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static partial bool IsWindow(nint hWnd);

    [LibraryImport(DllName)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static partial bool IsWindowVisible(nint hWnd);

    [LibraryImport(DllName)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static partial bool GetWindowRect(nint hWnd, out RECT lpRect);

    [LibraryImport(DllName)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static partial bool SetWindowPos(nint hWnd, nint hWndInsertAfter, int X, int Y, int cx, int cy, uint uFlags);

    [LibraryImport(DllName)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static partial bool GetCursorPos(out POINT lpPoint);

    [LibraryImport(DllName)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static partial bool SetCursorPos(int X, int Y);

    [LibraryImport(DllName)]
    public static partial nint MonitorFromWindow(nint hwnd, int dwFlags);

    [LibraryImport(DllName)]
    public static partial nint MonitorFromPoint(POINT pt, int dwFlags);

    [LibraryImport(DllName, EntryPoint = "GetMonitorInfoW")]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static partial bool GetMonitorInfo(nint hMonitor, ref MONITORINFO lpmi);

    [LibraryImport(DllName)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static partial bool EnumDisplayMonitors(nint hdc, nint lprcClip, EnumMonitorsDelegate lpfnEnum, nint dwData);

    public delegate bool EnumMonitorsDelegate(nint hMonitor, nint hdcMonitor, ref RECT lprcMonitor, nint dwData);

    [LibraryImport(DllName)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static partial bool EnumWindows(EnumWindowsDelegate lpEnumFunc, nint lParam);

    public delegate bool EnumWindowsDelegate(nint hWnd, nint lParam);

    [LibraryImport(DllName)]
    public static partial uint GetWindowThreadProcessId(nint hWnd, out uint lpdwProcessId);

    [LibraryImport(DllName)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static partial bool SetForegroundWindow(nint hWnd);

    [LibraryImport(DllName)]
    public static partial int ShowWindow(nint hWnd, int nCmdShow);

    public const int SW_RESTORE = 9;
    public const int SW_SHOW = 5;
}
