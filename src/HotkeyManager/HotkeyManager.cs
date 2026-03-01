using System.Collections.Concurrent;
using System.Runtime.InteropServices;
using System.Linq;
using System.Runtime.Versioning;
using Interop;

namespace HotkeyManager;

[SupportedOSPlatform("windows")]

/// <summary>Global hotkeys via low-level keyboard hook (and optional RegisterHotKey). Raises HotkeyPressed with the registered id.</summary>
public sealed class HotkeyManager : IDisposable
{
    private const int TrayCallbackMessage = User32Menu.WM_APP + 1;
    private const uint WM_HOOK_HOTKEY = User32Menu.WM_APP + 2;
    private const int TrayMenuIdQuit = 1;

    private nint _hwnd;
    private Thread? _thread;
    private volatile bool _running;
    private readonly ConcurrentDictionary<int, (uint mod, uint vk)> _registrations = new();
    private static readonly ConcurrentDictionary<(uint mod, uint vk), int> s_keyToIdForHook = new();
    private int _nextId = 1;
    private static HotkeyWin32.WndProc? s_wndProc;
    private static nint s_wndProcPtr;
    private static readonly object s_sync = new();
    private static nint s_hookHwnd;
    private static HotkeyWin32.LowLevelKeyboardProc? s_hookProc;
    private static nint s_hookProcPtr;
    private nint _hookHandle;
    private bool _trayAdded;
    private int _trayIconId = 1;
    private Action<string>? _log;

    /// <summary>Raised when a registered hotkey is pressed (on the hotkey thread). Marshal to UI thread if needed.</summary>
    public event Action<int>? HotkeyPressed;

    /// <summary>Raised when user chooses Quit from the tray menu. Marshal to UI thread to exit app.</summary>
    public event Action? TrayExitRequested;

    /// <summary>Raised when user double-clicks the tray icon. Marshal to UI thread to show/restore main window.</summary>
    public event Action? TrayShowWindowRequested;

    /// <summary>Optional diagnostic logging (e.g. "Hotkey window created", "WM_HOTKEY id=1").</summary>
    public void SetDiagnosticLog(Action<string>? log) => _log = log;

    public void Start()
    {
        if (_thread is { IsAlive: true }) return;
        _running = true;
        _thread = new Thread(MessageLoop) { IsBackground = true };
        _thread.SetApartmentState(ApartmentState.STA);
        _thread.Start();
        // Wait until window is created
        while (_hwnd == nint.Zero && _running)
            Thread.Sleep(10);
    }

    public void Stop()
    {
        _running = false;
        if (_hwnd != nint.Zero)
            HotkeyWin32.PostMessageW(_hwnd, HotkeyWin32.WM_CLOSE, nint.Zero, nint.Zero);
        _thread?.Join(2000);
        _hwnd = nint.Zero;
    }

    /// <summary>Result of Register. Id is set on success; ErrorCode is set on failure (Win32 GetLastError).</summary>
    public readonly record struct RegisterResult(int? Id, uint? ErrorCode);

    /// <summary>Register a hotkey. Uses low-level hook so it works even when RegisterHotKey is blocked.</summary>
    public RegisterResult Register(uint modifiers, uint virtualKey)
    {
        if (_hwnd == nint.Zero) return new RegisterResult(null, null);
        int id = Interlocked.Increment(ref _nextId);
        uint modKey = modifiers & ~HotkeyWin32.MOD_NOREPEAT;
        s_keyToIdForHook[(modKey, virtualKey)] = id;
        _registrations[id] = (modifiers, virtualKey);
        return new RegisterResult(id, null);
    }

    public void Unregister(int id)
    {
        if (_registrations.TryRemove(id, out var pair))
            s_keyToIdForHook.TryRemove((pair.mod & ~HotkeyWin32.MOD_NOREPEAT, pair.vk), out _);
    }

    /// <summary>Add a tray icon with the given tooltip. Call after Start(). Right-click shows Quit.</summary>
    public void AddTrayIcon(string tooltip = "RectangleWin")
    {
        if (_hwnd == nint.Zero) return;
        var nid = new Shell32.NOTIFYICONDATAW
        {
            cbSize = Marshal.SizeOf<Shell32.NOTIFYICONDATAW>(),
            hWnd = _hwnd,
            uID = _trayIconId,
            uFlags = Shell32.NIF_MESSAGE | Shell32.NIF_TIP,
            uCallbackMessage = TrayCallbackMessage,
            szTip = tooltip.Length > 127 ? tooltip[..127] : tooltip
        };
        nid.hIcon = User32Menu.LoadIcon(nint.Zero, (nint)User32Menu.IDI_APPLICATION);
        if (nid.hIcon != nint.Zero)
            nid.uFlags |= Shell32.NIF_ICON;

        if (Shell32.Shell_NotifyIconW(Shell32.NIM_ADD, ref nid))
            _trayAdded = true;
    }

    public void Dispose() => Stop();

    private void MessageLoop()
    {
        lock (s_sync)
        {
            if (s_wndProc == null)
            {
                s_wndProc = WndProc;
                s_wndProcPtr = Marshal.GetFunctionPointerForDelegate(s_wndProc);
            }
        }

        string className = "RectangleWinHotkey_" + Guid.NewGuid().ToString("N")[..8];
        var wc = new HotkeyWin32.WNDCLASSEXW
        {
            cbSize = Marshal.SizeOf<HotkeyWin32.WNDCLASSEXW>(),
            style = (int)(HotkeyWin32.CS_HREDRAW | HotkeyWin32.CS_VREDRAW),
            lpfnWndProc = s_wndProcPtr,
            hInstance = Marshal.GetHINSTANCE(typeof(HotkeyManager).Module),
            lpszClassName = className
        };

        if (HotkeyWin32.RegisterClassExW(ref wc) == 0)
            return;

        _hwnd = HotkeyWin32.CreateWindowExW(
            HotkeyWin32.WS_EX_TOOLWINDOW,
            className,
            null,
            HotkeyWin32.WS_OVERLAPPED,
            0, 0, 0, 0,
            nint.Zero, nint.Zero, wc.hInstance, nint.Zero);

        if (_hwnd == nint.Zero)
            return;

        _log?.Invoke("Hotkey window created");
        s_hookHwnd = _hwnd;
        lock (s_sync)
        {
            if (s_hookProc == null)
            {
                s_hookProc = LowLevelKeyboardHookProc;
                s_hookProcPtr = Marshal.GetFunctionPointerForDelegate(s_hookProc);
            }
        }
        nint hMod = Marshal.GetHINSTANCE(typeof(HotkeyManager).Module);
        _hookHandle = HotkeyWin32.SetWindowsHookExW(HotkeyWin32.WH_KEYBOARD_LL, s_hookProc, hMod, 0);
        if (_hookHandle == nint.Zero)
            _log?.Invoke("WH_KEYBOARD_LL hook failed");
        else
            _log?.Invoke("Low-level keyboard hook installed");

        try
        {
            while (_running && HotkeyWin32.GetMessage(out var msg, nint.Zero, 0, 0))
            {
                HotkeyWin32.TranslateMessage(ref msg);
                HotkeyWin32.DispatchMessageW(ref msg);
            }
        }
        finally
        {
            if (_hookHandle != nint.Zero)
            {
                HotkeyWin32.UnhookWindowsHookEx(_hookHandle);
                _hookHandle = nint.Zero;
            }
            if (_trayAdded)
            {
                var nid = new Shell32.NOTIFYICONDATAW
                {
                    cbSize = Marshal.SizeOf<Shell32.NOTIFYICONDATAW>(),
                    hWnd = _hwnd,
                    uID = _trayIconId
                };
                Shell32.Shell_NotifyIconW(Shell32.NIM_DELETE, ref nid);
            }
            _registrations.Clear();
            foreach (var kv in s_keyToIdForHook.Keys.ToList())
                s_keyToIdForHook.TryRemove(kv, out _);
            HotkeyWin32.DestroyWindow(_hwnd);
            _hwnd = nint.Zero;
        }
    }

    private static nint LowLevelKeyboardHookProc(int nCode, nint wParam, nint lParam)
    {
        if (nCode != HotkeyWin32.HC_ACTION || (wParam != HotkeyWin32.WM_KEYDOWN && wParam != HotkeyWin32.WM_SYSKEYDOWN))
            return HotkeyWin32.CallNextHookEx(nint.Zero, nCode, wParam, lParam);
        var kb = Marshal.PtrToStructure<HotkeyWin32.KBDLLHOOKSTRUCT>(lParam);
        uint mod = 0;
        if ((HotkeyWin32.GetAsyncKeyState(HotkeyWin32.VK_CONTROL) & 0x8000) != 0) mod |= HotkeyWin32.MOD_CONTROL;
        if ((HotkeyWin32.GetAsyncKeyState(HotkeyWin32.VK_MENU) & 0x8000) != 0) mod |= HotkeyWin32.MOD_ALT;
        if ((HotkeyWin32.GetAsyncKeyState(HotkeyWin32.VK_SHIFT) & 0x8000) != 0) mod |= HotkeyWin32.MOD_SHIFT;
        if ((HotkeyWin32.GetAsyncKeyState(HotkeyWin32.VK_LWIN) & 0x8000) != 0 || (HotkeyWin32.GetAsyncKeyState(HotkeyWin32.VK_RWIN) & 0x8000) != 0)
            mod |= HotkeyWin32.MOD_WIN;
        if (s_keyToIdForHook.TryGetValue((mod, kb.vkCode), out int id) && s_hookHwnd != nint.Zero)
        {
            HotkeyWin32.PostMessageW(s_hookHwnd, WM_HOOK_HOTKEY, (nint)id, nint.Zero);
            return (nint)1; // Swallow key so Win+1 etc. don't trigger taskbar/Start
        }
        return HotkeyWin32.CallNextHookEx(nint.Zero, nCode, wParam, lParam);
    }

    private nint WndProc(nint hWnd, uint uMsg, nint wParam, nint lParam)
    {
        switch (uMsg)
        {
            case HotkeyWin32.WM_HOTKEY:
            case WM_HOOK_HOTKEY:
                int id = wParam.ToInt32();
                _log?.Invoke("Hotkey fired id=" + id);
                HotkeyPressed?.Invoke(id);
                return nint.Zero;
            case (uint)TrayCallbackMessage:
                int lParamMsg = lParam.ToInt32();
                if (lParamMsg == User32Menu.WM_LBUTTONUP || lParamMsg == User32Menu.WM_LBUTTONDBLCLK)
                    TrayShowWindowRequested?.Invoke();
                else if (lParamMsg == User32Menu.WM_RBUTTONUP)
                    ShowTrayMenu();
                return nint.Zero;
            case HotkeyWin32.WM_CLOSE:
                HotkeyWin32.DestroyWindow(hWnd);
                return nint.Zero;
            case HotkeyWin32.WM_DESTROY:
                HotkeyWin32.PostQuitMessage(0);
                return nint.Zero;
            default:
                return HotkeyWin32.DefWindowProcW(hWnd, uMsg, wParam, lParam);
        }
    }

    private void ShowTrayMenu()
    {
        nint menu = User32Menu.CreatePopupMenu();
        if (menu == nint.Zero) return;
        try
        {
            User32Menu.AppendMenuW(menu, User32Menu.MF_STRING, (nint)TrayMenuIdQuit, "Quit");
            User32Menu.GetCursorPos(out POINT pt);
            int cmd = User32Menu.TrackPopupMenuEx(menu, User32Menu.TPM_RIGHTALIGN | User32Menu.TPM_NONOTIFY | User32Menu.TPM_RETURNCMD, pt.X, pt.Y, _hwnd, nint.Zero);
            if (cmd == TrayMenuIdQuit)
                TrayExitRequested?.Invoke();
        }
        finally
        {
            User32Menu.DestroyMenu(menu);
        }
    }
}
