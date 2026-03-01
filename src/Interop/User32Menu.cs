using System.Runtime.InteropServices;

namespace Interop;

public static partial class User32Menu
{
    private const string DllName = "user32.dll";

    public const int WM_LBUTTONUP = 0x0202;
    public const int WM_LBUTTONDBLCLK = 0x0203;
    public const int WM_RBUTTONUP = 0x0205;
    public const int TPM_RIGHTALIGN = 0x0008;
    public const int TPM_NONOTIFY = 0x0080;
    public const int TPM_RETURNCMD = 0x0100;
    public const uint MF_STRING = 0;
    public const uint MF_SEPARATOR = 0x800;

    [DllImport(DllName, CharSet = CharSet.Unicode)]
    public static extern nint CreatePopupMenu();

    [DllImport(DllName, CharSet = CharSet.Unicode)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool AppendMenuW(nint hMenu, uint uFlags, nint uIDNewItem, [MarshalAs(UnmanagedType.LPWStr)] string? lpNewItem);

    [DllImport(DllName)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool DestroyMenu(nint hObject);

    [DllImport(DllName)]
    public static extern int TrackPopupMenuEx(nint hMenu, uint uFlags, int x, int y, nint hwnd, nint lptpm);

    [DllImport(DllName)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool GetCursorPos(out POINT lpPoint);

    public const int WM_APP = 0x8000;
    public const int IDI_APPLICATION = 32512;

    [DllImport(DllName)]
    public static extern nint LoadIcon(nint hInstance, nint lpIconName);

    [DllImport(DllName)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool DestroyIcon(nint hIcon);

    [DllImport(DllName, CharSet = CharSet.Unicode)]
    public static extern int MessageBoxW(nint hWnd, string lpText, string lpCaption, uint uType);

    public const uint MB_OK = 0;
    public const uint MB_ICONERROR = 0x10;
}
