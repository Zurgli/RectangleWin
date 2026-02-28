using System.Runtime.InteropServices;

namespace Interop;

internal static partial class DwmApi
{
    private const string DllName = "dwmapi.dll";

    public enum DWMWINDOWATTRIBUTE : uint
    {
        DWMWA_EXTENDED_FRAME_BOUNDS = 9,
        DWMWA_CLOAKED = 14,
    }

    [LibraryImport(DllName)]
    public static partial int DwmGetWindowAttribute(nint hwnd, DWMWINDOWATTRIBUTE dwAttribute, out RECT pvAttribute, int cbAttribute);

    [LibraryImport(DllName)]
    public static partial int DwmGetWindowAttribute(nint hwnd, DWMWINDOWATTRIBUTE dwAttribute, out int pvAttribute, int cbAttribute);
}
