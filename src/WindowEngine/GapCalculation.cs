namespace WindowEngine;

[Flags]
public enum Dimension
{
    None = 0,
    Horizontal = 1 << 0,
    Vertical = 1 << 1,
    Both = Horizontal | Vertical,
}

[Flags]
public enum Edge
{
    None = 0,
    Left = 1 << 0,
    Right = 1 << 1,
    Top = 1 << 2,
    Bottom = 1 << 3,
}

public static class GapCalculation
{
    /// <summary>Insets rect by gap; optionally gives back half on shared edges.</summary>
    public static Rect ApplyGaps(Rect rect, Dimension dimension, Edge sharedEdges, float gapSize)
    {
        int g = (int)gapSize;
        int half = (int)(gapSize / 2);
        int dx = dimension.HasFlag(Dimension.Horizontal) ? g : 0;
        int dy = dimension.HasFlag(Dimension.Vertical) ? g : 0;

        int l = rect.Left + dx - (dimension.HasFlag(Dimension.Horizontal) && sharedEdges.HasFlag(Edge.Left) ? half : 0);
        int t = rect.Top + dy - (dimension.HasFlag(Dimension.Vertical) && sharedEdges.HasFlag(Edge.Bottom) ? half : 0);
        int r = rect.Right - dx + (dimension.HasFlag(Dimension.Horizontal) && sharedEdges.HasFlag(Edge.Right) ? half : 0);
        int b = rect.Bottom - dy + (dimension.HasFlag(Dimension.Vertical) && sharedEdges.HasFlag(Edge.Top) ? half : 0);

        return new Rect(l, t, r, b);
    }
}
