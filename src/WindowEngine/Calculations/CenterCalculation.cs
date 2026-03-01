namespace WindowEngine.Calculations;

public sealed class CenterCalculation : BaseCalculation
{
    /// <summary>Moves the window to the center of the work area without changing its size.</summary>
    public override CalculationResult? Calculate(RectCalculationParameters parameters)
    {
        var win = parameters.WindowRect;
        var work = parameters.WorkArea;

        int x = work.Left + (work.Width - win.Width) / 2;
        int y = work.Top + (work.Height - win.Height) / 2;

        return new CalculationResult(new Rect(x, y, x + win.Width, y + win.Height), WindowAction.Center);
    }
}
