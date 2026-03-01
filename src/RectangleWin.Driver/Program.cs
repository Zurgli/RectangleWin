using Core;
using WindowEngine;

if (!OperatingSystem.IsWindows())
{
    Console.WriteLine("RectangleWin.Driver runs on Windows only.");
    return 1;
}

var manager = new WindowManager();
var options = new ExecuteOptions { GapSize = 0 };

// Transformer: hold Alt (driver); real app will use Win+Alt for global hotkeys
const string modifierHint = "Alt";
Console.WriteLine("RectangleWin.Driver - hold {0} and press key to move foreground window:", modifierHint);
Console.WriteLine("  1=LeftHalf  2=RightHalf  3=TopHalf  4=BottomHalf");
Console.WriteLine("  5=Maximize  6=Center  Q=UpperLeft  W=UpperRight  A=LowerLeft  S=LowerRight");
Console.WriteLine("  N=NextDisplay  P=PreviousDisplay  R=Undo");
Console.WriteLine("  (Esc = exit, no modifier)");
Console.WriteLine();

while (true)
{
    var key = Console.ReadKey(intercept: true);
    if (key.Key == ConsoleKey.Escape) break;

    // Only act when transformer modifier is held (Alt in console; Win+Alt in tray app)
    bool modifierHeld = (key.Modifiers & ConsoleModifiers.Alt) != 0;
    if (!modifierHeld)
        continue;

    WindowAction? action = key.Key switch
    {
        ConsoleKey.D1 => WindowAction.LeftHalf,
        ConsoleKey.D2 => WindowAction.RightHalf,
        ConsoleKey.D3 => WindowAction.TopHalf,
        ConsoleKey.D4 => WindowAction.BottomHalf,
        ConsoleKey.D5 => WindowAction.Maximize,
        ConsoleKey.D6 => WindowAction.Center,
        ConsoleKey.Q => WindowAction.UpperLeft,
        ConsoleKey.W => WindowAction.UpperRight,
        ConsoleKey.A => WindowAction.LowerLeft,
        ConsoleKey.S => WindowAction.LowerRight,
        ConsoleKey.N => WindowAction.NextDisplay,
        ConsoleKey.P => WindowAction.PreviousDisplay,
        ConsoleKey.R => WindowAction.Undo,
        _ => null
    };

    if (action is not { } a)
        continue;

    bool ok = manager.Execute(a, options: options);
    Console.WriteLine(ok ? "  {0}" : "  (no effect)", a);
}

return 0;
