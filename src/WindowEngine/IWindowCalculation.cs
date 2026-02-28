namespace WindowEngine;

public interface IWindowCalculation
{
    /// <summary>Returns target rect for the action, or null if not applicable.</summary>
    CalculationResult? Calculate(RectCalculationParameters parameters);
}
