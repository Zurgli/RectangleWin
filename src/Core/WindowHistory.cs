using Interop;
using WindowEngine;

namespace Core;

/// <summary>In-memory restore bounds and last action per window (keyed by HWND).</summary>
public sealed class WindowHistory
{
    private readonly Dictionary<nint, RECT> _restoreRects = new();
    private readonly Dictionary<nint, RectangleAction> _lastActions = new();

    public IReadOnlyDictionary<nint, RECT> RestoreRects => _restoreRects;
    public IReadOnlyDictionary<nint, RectangleAction> LastRectangleActions => _lastActions;

    public void SetRestoreRect(nint hwnd, RECT rect) => _restoreRects[hwnd] = rect;
    public RECT? GetRestoreRect(nint hwnd) => _restoreRects.TryGetValue(hwnd, out var r) ? r : null;
    public void RemoveRestoreRect(nint hwnd) => _restoreRects.Remove(hwnd);

    public void SetLastAction(nint hwnd, RectangleAction action) => _lastActions[hwnd] = action;
    public RectangleAction? GetLastAction(nint hwnd) => _lastActions.TryGetValue(hwnd, out var a) ? a : null;
    public void RemoveLastAction(nint hwnd) => _lastActions.Remove(hwnd);
}

public readonly record struct RectangleAction(WindowAction Action, RECT Rect);
