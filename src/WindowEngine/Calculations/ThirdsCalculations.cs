namespace WindowEngine.Calculations;

/// <summary>Left 1/3 of work area (full height).</summary>
public sealed class FirstThirdCalculation : BaseCalculation
{
    public override CalculationResult? Calculate(RectCalculationParameters parameters)
    {
        var w = parameters.WorkArea;
        int width = w.Width;
        int third = width / 3;
        var rect = new Rect(w.Left, w.Top, w.Left + third, w.Bottom);
        return new CalculationResult(rect, parameters.Action);
    }
}

/// <summary>Left 2/3 of work area (full height). Uses same split as CenterThird/LastThird so no gap.</summary>
public sealed class FirstTwoThirdsCalculation : BaseCalculation
{
    public override CalculationResult? Calculate(RectCalculationParameters parameters)
    {
        var w = parameters.WorkArea;
        int third = w.Width / 3;
        var rect = new Rect(w.Left, w.Top, w.Left + 2 * third, w.Bottom);
        return new CalculationResult(rect, parameters.Action);
    }
}

/// <summary>Center 1/3 of work area (full height).</summary>
public sealed class CenterThirdCalculation : BaseCalculation
{
    public override CalculationResult? Calculate(RectCalculationParameters parameters)
    {
        var w = parameters.WorkArea;
        int width = w.Width;
        int third = width / 3;
        var rect = new Rect(w.Left + third, w.Top, w.Left + 2 * third, w.Bottom);
        return new CalculationResult(rect, parameters.Action);
    }
}

/// <summary>Right 2/3 of work area (full height). Uses same split as FirstThird so no gap.</summary>
public sealed class LastTwoThirdsCalculation : BaseCalculation
{
    public override CalculationResult? Calculate(RectCalculationParameters parameters)
    {
        var w = parameters.WorkArea;
        int third = w.Width / 3;
        var rect = new Rect(w.Left + third, w.Top, w.Right, w.Bottom);
        return new CalculationResult(rect, parameters.Action);
    }
}

/// <summary>Right 1/3 of work area (full height). Uses same split as CenterThird so no gap.</summary>
public sealed class LastThirdCalculation : BaseCalculation
{
    public override CalculationResult? Calculate(RectCalculationParameters parameters)
    {
        var w = parameters.WorkArea;
        int third = w.Width / 3;
        var rect = new Rect(w.Left + 2 * third, w.Top, w.Right, w.Bottom);
        return new CalculationResult(rect, parameters.Action);
    }
}

/// <summary>Center 2/3 of work area (full height).</summary>
public sealed class CenterTwoThirdsCalculation : BaseCalculation
{
    public override CalculationResult? Calculate(RectCalculationParameters parameters)
    {
        var w = parameters.WorkArea;
        int width = w.Width;
        int sixth = width / 6;
        var rect = new Rect(w.Left + sixth, w.Top, w.Left + 5 * sixth, w.Bottom);
        return new CalculationResult(rect, parameters.Action);
    }
}
