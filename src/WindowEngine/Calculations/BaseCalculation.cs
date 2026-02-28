namespace WindowEngine.Calculations;

public abstract class BaseCalculation : IWindowCalculation
{
    public abstract CalculationResult? Calculate(RectCalculationParameters parameters);

    protected static bool IsRepeatedCommand(RectCalculationParameters p)
    {
        if (p.LastAction is not { } last) return false;
        if (last.Action != p.Action) return false;
        return RectsEqual(last.Rect, p.WindowRect);
    }

    protected static bool RectsEqual(Rect a, Rect b) =>
        a.Left == b.Left && a.Top == b.Top && a.Right == b.Right && a.Bottom == b.Bottom;
}
