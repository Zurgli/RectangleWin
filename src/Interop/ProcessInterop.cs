using System.Text;

namespace Interop;

/// <summary>Public API for process identity (disabled-apps list).</summary>
public static class ProcessInterop
{
    /// <summary>Gets the process image file name (full path or exe name) for the window's process. Returns null on failure.</summary>
    public static string? GetProcessImageFileName(nint hwnd)
    {
        if (hwnd == nint.Zero) return null;
        User32.GetWindowThreadProcessId(hwnd, out uint pid);
        if (pid == 0) return null;
        nint hProcess = Kernel32.OpenProcess(Kernel32.PROCESS_QUERY_LIMITED_INFORMATION, false, pid);
        if (hProcess == nint.Zero) return null;
        try
        {
            var sb = new StringBuilder(260);
            int size = sb.Capacity;
            if (!Kernel32.QueryFullProcessImageName(hProcess, Kernel32.PROCESS_NAME_NATIVE, sb, ref size))
                return null;
            string path = sb.ToString();
            return string.IsNullOrEmpty(path) ? null : Path.GetFileName(path);
        }
        finally
        {
            Kernel32.CloseHandle(hProcess);
        }
    }
}
