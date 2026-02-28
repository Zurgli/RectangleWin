using System.Runtime.InteropServices;

namespace Interop;

/// <summary>Win32 RECT (left, top, right, bottom) in screen coordinates.</summary>
[StructLayout(LayoutKind.Sequential)]
public struct RECT
{
    public int Left;
    public int Top;
    public int Right;
    public int Bottom;

    public int Width => Right - Left;
    public int Height => Bottom - Top;
    public bool IsEmpty => Left == 0 && Top == 0 && Right == 0 && Bottom == 0;
}

/// <summary>Win32 POINT.</summary>
[StructLayout(LayoutKind.Sequential)]
public struct POINT
{
    public int X;
    public int Y;
}

/// <summary>Win32 MONITORINFO; cbSize must be set before passing to GetMonitorInfo.</summary>
[StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
public struct MONITORINFO
{
    public int cbSize;
    public RECT rcMonitor;
    public RECT rcWork;
    public uint dwFlags;

    public static MONITORINFO Create() => new() { cbSize = Marshal.SizeOf<MONITORINFO>() };
}

/// <summary>MONITORINFOEX for EnumDisplayMonitors (includes device name).</summary>
[StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
public struct MONITORINFOEX
{
    public int cbSize;
    public RECT rcMonitor;
    public RECT rcWork;
    public uint dwFlags;
    [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)]
    public string szDevice;

    public static MONITORINFOEX Create() => new() { cbSize = Marshal.SizeOf<MONITORINFOEX>(), szDevice = "" };
}
