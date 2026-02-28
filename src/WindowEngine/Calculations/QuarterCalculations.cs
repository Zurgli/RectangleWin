namespace WindowEngine.Calculations;

/// <summary>Lower-left quarter of work area.</summary>
public sealed class LowerLeftCalculation : BaseCalculation
{
    public override CalculationResult? Calculate(RectCalculationParameters parameters)
    {
        var w = parameters.WorkArea;
        int hw = w.Width / 2, hh = w.Height / 2;
        var rect = new Rect(w.Left, w.Top + hh, w.Left + hw, w.Bottom);
        return new CalculationResult(rect, parameters.Action);
    }
}

public sealed class LowerRightCalculation : BaseCalculation
{
    public override CalculationResult? Calculate(RectCalculationParameters parameters)
    {
        var w = parameters.WorkArea;
        int hw = w.Width / 2, hh = w.Height / 2;
        var rect = new Rect(w.Left + hw, w.Top + hh, w.Right, w.Bottom);
        return new CalculationResult(rect, parameters.Action);
    }
}

public sealed class UpperLeftCalculation : BaseCalculation
{
    public override CalculationResult? Calculate(RectCalculationParameters parameters)
    {
        var w = parameters.WorkArea;
        int hw = w.Width / 2, hh = w.Height / 2;
        var rect = new Rect(w.Left, w.Top, w.Left + hw, w.Top + hh);
        return new CalculationResult(rect, parameters.Action);
    }
}

public sealed class UpperRightCalculation : BaseCalculation
{
    public override CalculationResult? Calculate(RectCalculationParameters parameters)
    {
        var w = parameters.WorkArea;
        int hw = w.Width / 2, hh = w.Height / 2;
        var rect = new Rect(w.Left + hw, w.Top, w.Right, w.Top + hh);
        return new CalculationResult(rect, parameters.Action);
    }
}
