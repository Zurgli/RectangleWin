using WindowEngine.Calculations;

namespace WindowEngine;

public static class WindowCalculationFactory
{
    private static readonly IWindowCalculation LeftHalf = new LeftRightHalfCalculation(false);
    private static readonly IWindowCalculation RightHalf = new LeftRightHalfCalculation(true);
    private static readonly IWindowCalculation TopHalf = new TopBottomHalfCalculation(false);
    private static readonly IWindowCalculation BottomHalf = new TopBottomHalfCalculation(true);
    private static readonly IWindowCalculation Maximize = new MaximizeCalculation();
    private static readonly IWindowCalculation Center = new CenterCalculation();
    private static readonly IWindowCalculation LowerLeft = new LowerLeftCalculation();
    private static readonly IWindowCalculation LowerRight = new LowerRightCalculation();
    private static readonly IWindowCalculation UpperLeft = new UpperLeftCalculation();
    private static readonly IWindowCalculation UpperRight = new UpperRightCalculation();
    private static readonly IWindowCalculation FirstThird = new FirstThirdCalculation();
    private static readonly IWindowCalculation FirstTwoThirds = new FirstTwoThirdsCalculation();
    private static readonly IWindowCalculation CenterThird = new CenterThirdCalculation();
    private static readonly IWindowCalculation LastTwoThirds = new LastTwoThirdsCalculation();
    private static readonly IWindowCalculation LastThird = new LastThirdCalculation();
    private static readonly IWindowCalculation CenterTwoThirds = new CenterTwoThirdsCalculation();

    private static readonly Dictionary<WindowAction, IWindowCalculation> Map = new()
    {
        [WindowAction.LeftHalf] = LeftHalf,
        [WindowAction.RightHalf] = RightHalf,
        [WindowAction.TopHalf] = TopHalf,
        [WindowAction.BottomHalf] = BottomHalf,
        [WindowAction.Maximize] = Maximize,
        [WindowAction.Center] = Center,
        [WindowAction.LowerLeft] = LowerLeft,
        [WindowAction.LowerRight] = LowerRight,
        [WindowAction.UpperLeft] = UpperLeft,
        [WindowAction.UpperRight] = UpperRight,
        [WindowAction.FirstThird] = FirstThird,
        [WindowAction.FirstTwoThirds] = FirstTwoThirds,
        [WindowAction.CenterThird] = CenterThird,
        [WindowAction.LastTwoThirds] = LastTwoThirds,
        [WindowAction.LastThird] = LastThird,
        [WindowAction.CenterTwoThirds] = CenterTwoThirds,
    };

    public static IWindowCalculation? GetCalculation(WindowAction action) =>
        Map.TryGetValue(action, out var calc) ? calc : null;

    /// <summary>Whether this action uses a layout calculation (vs Undo or Next/Prev which are handled by manager).</summary>
    public static bool HasCalculation(WindowAction action) => action != WindowAction.Undo && action != WindowAction.NextDisplay && action != WindowAction.PreviousDisplay;
}
