using Interop;
using WindowEngine;

namespace Core;

internal static class RectConversion
{
    public static Rect ToEngine(this RECT r) => new(r.Left, r.Top, r.Right, r.Bottom);
    public static RECT ToInterop(this Rect r) => new RECT { Left = r.Left, Top = r.Top, Right = r.Right, Bottom = r.Bottom };
}
