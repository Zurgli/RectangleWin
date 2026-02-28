namespace WindowEngine.Calculations;

public sealed class TopBottomHalfCalculation : BaseCalculation
{
    private readonly bool _bottomHalf;

    public TopBottomHalfCalculation(bool bottomHalf) => _bottomHalf = bottomHalf;

    public override CalculationResult? Calculate(RectCalculationParameters parameters)
    {
        var w = parameters.WorkArea;
        int height = w.Height / 2;
        Rect rect = _bottomHalf
            ? new Rect(w.Left, w.Top + height, w.Right, w.Bottom)
            : new Rect(w.Left, w.Top, w.Right, w.Top + height);
        return new CalculationResult(rect, parameters.Action);
    }
}
