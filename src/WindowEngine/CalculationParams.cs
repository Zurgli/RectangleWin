namespace WindowEngine;

public readonly record struct RectCalculationParameters(
    Rect WindowRect,
    Rect WorkArea,
    WindowAction Action,
    LastActionInfo? LastAction);

public readonly record struct LastActionInfo(Rect Rect, WindowAction Action);

public readonly record struct CalculationResult(Rect Rect, WindowAction ResultingAction);
