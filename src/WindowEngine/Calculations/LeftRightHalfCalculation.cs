namespace WindowEngine.Calculations;

public sealed class LeftRightHalfCalculation : BaseCalculation
{
    private readonly bool _rightHalf;

    public LeftRightHalfCalculation(bool rightHalf) => _rightHalf = rightHalf;

    public override CalculationResult? Calculate(RectCalculationParameters parameters)
    {
        var w = parameters.WorkArea;
        int width = w.Width / 2;
        Rect rect = _rightHalf
            ? new Rect(w.Left + width, w.Top, w.Right, w.Bottom)
            : new Rect(w.Left, w.Top, w.Left + width, w.Bottom);
        return new CalculationResult(rect, parameters.Action);
    }
}
