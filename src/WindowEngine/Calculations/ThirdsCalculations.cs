namespace WindowEngine.Calculations;

internal static class ThirdsLayout
{
    public static bool IsFifths(RectCalculationParameters p) =>
        string.Equals(p.ThirdsLayoutMode, "Fifths", StringComparison.OrdinalIgnoreCase);
}

/// <summary>Left 1/3 of work area (full height). Fifths: left 1/5.</summary>
public sealed class FirstThirdCalculation : BaseCalculation
{
    public override CalculationResult? Calculate(RectCalculationParameters parameters)
    {
        var w = parameters.WorkArea;
        int width = w.Width;
        if (ThirdsLayout.IsFifths(parameters))
        {
            int unit = width / 5;
            var rect = new Rect(w.Left, w.Top, w.Left + unit, w.Bottom);
            return new CalculationResult(rect, parameters.Action);
        }
        int third = width / 3;
        var rect3 = new Rect(w.Left, w.Top, w.Left + third, w.Bottom);
        return new CalculationResult(rect3, parameters.Action);
    }
}

/// <summary>Left 2/3 of work area (full height). Fifths: left 4/5.</summary>
public sealed class FirstTwoThirdsCalculation : BaseCalculation
{
    public override CalculationResult? Calculate(RectCalculationParameters parameters)
    {
        var w = parameters.WorkArea;
        int width = w.Width;
        if (ThirdsLayout.IsFifths(parameters))
        {
            int unit = width / 5;
            var rect = new Rect(w.Left, w.Top, w.Left + 4 * unit, w.Bottom);
            return new CalculationResult(rect, parameters.Action);
        }
        int third = width / 3;
        var rect3 = new Rect(w.Left, w.Top, w.Left + 2 * third, w.Bottom);
        return new CalculationResult(rect3, parameters.Action);
    }
}

/// <summary>Center 1/3 of work area (full height). Fifths: center 3/5.</summary>
public sealed class CenterThirdCalculation : BaseCalculation
{
    public override CalculationResult? Calculate(RectCalculationParameters parameters)
    {
        var w = parameters.WorkArea;
        int width = w.Width;
        if (ThirdsLayout.IsFifths(parameters))
        {
            int unit = width / 5;
            var rect = new Rect(w.Left + unit, w.Top, w.Left + 4 * unit, w.Bottom);
            return new CalculationResult(rect, parameters.Action);
        }
        int third = width / 3;
        var rect3 = new Rect(w.Left + third, w.Top, w.Left + 2 * third, w.Bottom);
        return new CalculationResult(rect3, parameters.Action);
    }
}

/// <summary>Right 2/3 of work area (full height). Fifths: right 4/5.</summary>
public sealed class LastTwoThirdsCalculation : BaseCalculation
{
    public override CalculationResult? Calculate(RectCalculationParameters parameters)
    {
        var w = parameters.WorkArea;
        int width = w.Width;
        if (ThirdsLayout.IsFifths(parameters))
        {
            int unit = width / 5;
            var rect = new Rect(w.Left + unit, w.Top, w.Right, w.Bottom);
            return new CalculationResult(rect, parameters.Action);
        }
        int third = width / 3;
        var rect3 = new Rect(w.Left + third, w.Top, w.Right, w.Bottom);
        return new CalculationResult(rect3, parameters.Action);
    }
}

/// <summary>Right 1/3 of work area (full height). Fifths: right 1/5.</summary>
public sealed class LastThirdCalculation : BaseCalculation
{
    public override CalculationResult? Calculate(RectCalculationParameters parameters)
    {
        var w = parameters.WorkArea;
        int width = w.Width;
        if (ThirdsLayout.IsFifths(parameters))
        {
            int unit = width / 5;
            var rect = new Rect(w.Left + 4 * unit, w.Top, w.Right, w.Bottom);
            return new CalculationResult(rect, parameters.Action);
        }
        int third = width / 3;
        var rect3 = new Rect(w.Left + 2 * third, w.Top, w.Right, w.Bottom);
        return new CalculationResult(rect3, parameters.Action);
    }
}

/// <summary>Center 2/3 of work area (full height). Fifths: center 4/5.</summary>
public sealed class CenterTwoThirdsCalculation : BaseCalculation
{
    public override CalculationResult? Calculate(RectCalculationParameters parameters)
    {
        var w = parameters.WorkArea;
        int width = w.Width;
        if (ThirdsLayout.IsFifths(parameters))
        {
            int tenth = width / 10;
            var rect = new Rect(w.Left + tenth, w.Top, w.Left + 9 * tenth, w.Bottom);
            return new CalculationResult(rect, parameters.Action);
        }
        int sixth = width / 6;
        var rect3 = new Rect(w.Left + sixth, w.Top, w.Left + 5 * sixth, w.Bottom);
        return new CalculationResult(rect3, parameters.Action);
    }
}
