namespace WindowEngine;

public readonly record struct RectCalculationParameters(
    Rect WindowRect,
    Rect WorkArea,
    WindowAction Action,
    LastActionInfo? LastAction,
    string ThirdsLayoutMode = "Thirds");

public readonly record struct LastActionInfo(Rect Rect, WindowAction Action);

public readonly record struct CalculationResult(Rect Rect, WindowAction ResultingAction);
