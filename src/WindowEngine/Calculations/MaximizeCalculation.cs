namespace WindowEngine.Calculations;

public sealed class MaximizeCalculation : BaseCalculation
{
    public override CalculationResult? Calculate(RectCalculationParameters parameters) =>
        new CalculationResult(parameters.WorkArea, WindowAction.Maximize);
}
