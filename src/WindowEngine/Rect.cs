namespace WindowEngine;

/// <summary>Platform-agnostic rectangle (no Win32 types).</summary>
public readonly struct Rect
{
    public int Left { get; }
    public int Top { get; }
    public int Right { get; }
    public int Bottom { get; }

    public int Width => Right - Left;
    public int Height => Bottom - Top;

    public Rect(int left, int top, int right, int bottom)
    {
        Left = left;
        Top = top;
        Right = right;
        Bottom = bottom;
    }

    public static Rect Empty => new(0, 0, 0, 0);
    public bool IsEmpty => Left == 0 && Top == 0 && Right == 0 && Bottom == 0;
}
