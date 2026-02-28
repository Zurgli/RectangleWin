namespace WindowEngine.Calculations;

public sealed class CenterCalculation : BaseCalculation
{
    public override CalculationResult? Calculate(RectCalculationParameters parameters)
    {
        var win = parameters.WindowRect;
        var work = parameters.WorkArea;

        if (win.Width > work.Width && win.Height > work.Height)
            return new CalculationResult(work, WindowAction.Maximize);

        int x = win.Left, y = win.Top;
        int w = win.Width, h = win.Height;

        if (win.Width > work.Width)
        {
            w = work.Width;
            x = work.Left;
        }
        else
            x = work.Left + (work.Width - win.Width) / 2;

        if (win.Height > work.Height)
        {
            h = work.Height;
            y = work.Top;
        }
        else
            y = work.Top + (work.Height - win.Height) / 2;

        return new CalculationResult(new Rect(x, y, x + w, y + h), WindowAction.Center);
    }
}
