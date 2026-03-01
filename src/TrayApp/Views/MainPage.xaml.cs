using Microsoft.UI.Dispatching;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Controls.Primitives;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Media.Imaging;

namespace TrayApp.Views;

public sealed partial class MainPage : Page
{
    public MainPage()
    {
        InitializeComponent();
        Loaded += OnLoaded;
    }

    private void OnLoaded(object sender, RoutedEventArgs e)
    {
        if (App.MainWindow is { } window)
        {
            window.SetTitleBar(TitleBarPanel);
        }

        // Set title bar icon from app directory (reliable for unpackaged)
        try
        {
            var baseDir = AppContext.BaseDirectory;
            var iconPath = Path.Combine(baseDir, "Assets", "StoreLogo.png");
            if (File.Exists(iconPath))
            {
                var fullPath = Path.GetFullPath(iconPath);
                TitleBarIcon.Source = new BitmapImage(new Uri("file:///" + fullPath.Replace('\\', '/')));
            }
        }
        catch { /* ignore */ }

        if (App.Logic is not { } logic) return;

        // Defer clearing focus so it runs after WinUI's default focus (which highlights the first control)
        var queue = DispatcherQueue.GetForCurrentThread();
        queue.TryEnqueue(DispatcherQueuePriority.Low, () =>
        {
            try
            {
                if (FocusManager.GetFocusedElement(XamlRoot) is Control focused && focused != this)
                    _ = this.Focus(FocusState.Programmatic);
            }
            catch { /* ignore */ }
        });

        logic.HotkeyTriggered += OnHotkeyTriggered;

        var config = logic.Config;
        LaunchOnLoginCheckBox.IsOn = config.LaunchOnLogin;
        GapSlider.Value = config.GapSize;
        UpdateGapLabel(config.GapSize);

        var grouped = GroupHotkeysBySection(config.Hotkeys);
        void AddSections(StackPanel panel, IEnumerable<(string Title, List<(string Action, string Shortcut)> Items)> sections)
        {
            panel.Children.Clear();
            foreach (var (title, items) in sections)
            {
                if (items.Count == 0) continue;
                var sectionStack = new StackPanel { Spacing = 6, Margin = new Thickness(0, 0, 0, 14) };
                sectionStack.Children.Add(new TextBlock
                {
                    Text = title,
                    FontSize = 11,
                    Foreground = GetThemeBrush("TextFillColorSecondaryBrush") ?? new SolidColorBrush(Windows.UI.Color.FromArgb(255, 100, 100, 100)),
                    Margin = new Thickness(0, 0, 0, 2)
                });
                var rowsPanel = new StackPanel { Spacing = 2 };
                foreach (var (action, shortcut) in items)
                {
                    var row = new Grid
                    {
                        ColumnSpacing = 8,
                        VerticalAlignment = VerticalAlignment.Center,
                        Margin = new Thickness(0, 2, 0, 2)
                    };
                    row.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
                    row.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(0, GridUnitType.Auto) });
                    var left = new StackPanel
                    {
                        Orientation = Orientation.Horizontal,
                        Spacing = 8,
                        VerticalAlignment = VerticalAlignment.Center
                    };
                    left.Children.Add(TileIconForAction(action));
                    left.Children.Add(new TextBlock { Text = action, FontSize = 12, VerticalAlignment = VerticalAlignment.Center });
                    var shortcutBlock = new TextBlock
                    {
                        Text = shortcut,
                        FontSize = 12,
                        VerticalAlignment = VerticalAlignment.Center,
                        HorizontalAlignment = HorizontalAlignment.Right
                    };
                    Grid.SetColumn(left, 0);
                    Grid.SetColumn(shortcutBlock, 1);
                    row.Children.Add(left);
                    row.Children.Add(shortcutBlock);
                    rowsPanel.Children.Add(row);
                }
                sectionStack.Children.Add(rowsPanel);
                panel.Children.Add(sectionStack);
            }
        }
        AddSections(ShortcutsListLeft, new[] { grouped.Halves, grouped.Quarters });
        AddSections(ShortcutsListRight, new[] { grouped.Thirds, grouped.Other });
    }

    private void OnHotkeyTriggered(string actionName)
    {
        LastHotkeyText.Text = $"{actionName}  ({DateTime.Now:HH:mm:ss})";
    }

    private void OnLaunchOnLoginToggled(object sender, RoutedEventArgs e)
    {
        if (App.Logic is not { } logic) return;
        logic.Config.LaunchOnLogin = LaunchOnLoginCheckBox.IsOn;
        logic.SaveConfig();
        StartupRegistration.SetLaunchAtStartup(logic.Config.LaunchOnLogin);
    }

    private void OnGapValueChanged(object sender, RangeBaseValueChangedEventArgs e)
    {
        if (App.Logic is not { } logic) return;
        var v = (float)e.NewValue;
        logic.Config.GapSize = v;
        logic.SaveConfig();
        UpdateGapLabel(v);
    }

    private void UpdateGapLabel(float px)
    {
        GapValueText.Text = $"{(int)px} px";
    }

    private void OnMinimizeClick(object sender, RoutedEventArgs e)
    {
        // Minimize to tray: hide window; double-click tray icon shows it again
        App.MainWindow?.AppWindow?.Hide();
    }

    private void OnQuitClick(object sender, RoutedEventArgs e)
    {
        App.MainWindow?.Close();
    }

    private static ((string Title, List<(string Action, string Shortcut)> Items) Halves, (string, List<(string Action, string Shortcut)>) Quarters, (string, List<(string Action, string Shortcut)>) Thirds, (string, List<(string Action, string Shortcut)>) Other) GroupHotkeysBySection(List<HotkeyBinding> hotkeys)
    {
        var actionToShortcut = hotkeys.ToDictionary(h => h.Action, h => (h.Action, Shortcut: FormatShortcut(h.Modifiers, h.VirtualKey)));

        (string Title, string[] Actions)[] sectionDefs =
        {
            ("Halves", new[] { "LeftHalf", "RightHalf", "TopHalf", "BottomHalf" }),
            ("Quarters", new[] { "UpperLeft", "UpperRight", "LowerLeft", "LowerRight" }),
            ("Thirds", new[] { "FirstThird", "FirstTwoThirds", "CenterThird", "LastTwoThirds", "LastThird", "CenterTwoThirds" }),
            ("Other", new[] { "Maximize", "Center", "Undo", "NextDisplay", "PreviousDisplay" }),
        };

        var halves = new List<(string Action, string Shortcut)>();
        var quarters = new List<(string Action, string Shortcut)>();
        var thirds = new List<(string Action, string Shortcut)>();
        var other = new List<(string Action, string Shortcut)>();

        foreach (var (title, actions) in sectionDefs)
        {
            var list = title == "Halves" ? halves : title == "Quarters" ? quarters : title == "Thirds" ? thirds : other;
            foreach (var action in actions)
            {
                if (actionToShortcut.TryGetValue(action, out var pair))
                    list.Add(pair);
                else if (action == "Undo" && actionToShortcut.TryGetValue("Restore", out pair))
                    list.Add(("Undo", pair.Shortcut));
            }
        }

        return (
            Halves: ("Halves", halves),
            Quarters: ("Quarters", quarters),
            Thirds: ("Thirds", thirds),
            Other: ("Other", other)
        );
    }

    private static FrameworkElement TileIconForAction(string action)
    {
        const int size = 22;
        // Lighter grey so the small screen outline is clearly visible
        var stroke = new SolidColorBrush(Windows.UI.Color.FromArgb(255, 175, 175, 175));
        var fill = GetThemeBrush("AccentFillColorDefaultBrush") ?? new SolidColorBrush(Windows.UI.Color.FromArgb(255, 0, 120, 212));

        var screen = new Border
        {
            Width = size,
            Height = size,
            BorderBrush = stroke,
            BorderThickness = new Thickness(1),
            CornerRadius = new CornerRadius(1),
            Child = BuildTileGrid(action, size - 2, fill)
        };
        return screen;
    }

    private static Brush? GetThemeBrush(string key)
    {
        if (Application.Current.Resources.TryGetValue(key, out var v) && v is Brush b)
            return b;
        return null;
    }

    private static Grid BuildTileGrid(string action, int innerSize, Brush windowFill)
    {
        // Margin 0,0,2,2 shifts inner rect 1px up and 1px left for even gap from outline
        var window = new Border
        {
            Background = windowFill,
            CornerRadius = new CornerRadius(0),
            Margin = new Thickness(1, 1, 1.5, 1.5)
        };

        var grid = new Grid();
        action = action.Trim();

        // 2x2 for quarters and halves; 3x3 for thirds/center
        if (action is "LeftHalf" or "RightHalf")
        {
            grid.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
            grid.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
            grid.RowDefinitions.Add(new RowDefinition { Height = new GridLength(1, GridUnitType.Star) });
            Grid.SetColumn(window, action == "LeftHalf" ? 0 : 1);
            Grid.SetColumnSpan(window, 1);
        }
        else if (action is "TopHalf" or "BottomHalf")
        {
            grid.RowDefinitions.Add(new RowDefinition { Height = new GridLength(1, GridUnitType.Star) });
            grid.RowDefinitions.Add(new RowDefinition { Height = new GridLength(1, GridUnitType.Star) });
            grid.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
            Grid.SetRow(window, action == "TopHalf" ? 0 : 1);
            Grid.SetRowSpan(window, 1);
        }
        else if (action is "UpperLeft" or "UpperRight" or "LowerLeft" or "LowerRight")
        {
            grid.RowDefinitions.Add(new RowDefinition { Height = new GridLength(1, GridUnitType.Star) });
            grid.RowDefinitions.Add(new RowDefinition { Height = new GridLength(1, GridUnitType.Star) });
            grid.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
            grid.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
            int col = action.Contains("Right") ? 1 : 0;
            int row = action.Contains("Lower") ? 1 : 0;
            Grid.SetColumn(window, col);
            Grid.SetRow(window, row);
        }
        else if (action == "Maximize")
        {
            grid.RowDefinitions.Add(new RowDefinition { Height = new GridLength(1, GridUnitType.Star) });
            grid.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
            Grid.SetColumnSpan(window, 1);
        }
        else if (action == "Center" || action == "Undo")
        {
            for (int i = 0; i < 3; i++) { grid.RowDefinitions.Add(new RowDefinition { Height = new GridLength(1, GridUnitType.Star) }); grid.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) }); }
            Grid.SetRow(window, 1);
            Grid.SetColumn(window, 1);
            Grid.SetRowSpan(window, 1);
            Grid.SetColumnSpan(window, 1);
        }
        else if (action == "FirstThird" || action == "LastThird" || action == "FirstTwoThirds" || action == "LastTwoThirds" || action == "CenterThird" || action == "CenterTwoThirds")
        {
            for (int i = 0; i < 3; i++) { grid.RowDefinitions.Add(new RowDefinition { Height = new GridLength(1, GridUnitType.Star) }); grid.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) }); }
            int col, span;
            if (action == "FirstThird") { col = 0; span = 1; }
            else if (action == "LastThird") { col = 2; span = 1; }
            else if (action == "FirstTwoThirds") { col = 0; span = 2; }
            else if (action == "LastTwoThirds") { col = 1; span = 2; }
            else if (action == "CenterThird") { col = 1; span = 1; }
            else { col = 1; span = 2; } // CenterTwoThirds
            Grid.SetColumn(window, col);
            Grid.SetColumnSpan(window, span);
            Grid.SetRow(window, 0);
            Grid.SetRowSpan(window, 3);
        }
        else if (action == "NextDisplay" || action == "PreviousDisplay")
        {
            grid.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
            grid.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
            grid.RowDefinitions.Add(new RowDefinition { Height = new GridLength(1, GridUnitType.Star) });
            Grid.SetColumn(window, action == "NextDisplay" ? 1 : 0);
            Grid.SetColumnSpan(window, 1);
        }
        else
        {
            grid.RowDefinitions.Add(new RowDefinition { Height = new GridLength(1, GridUnitType.Star) });
            grid.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        }

        grid.Children.Add(window);
        return grid;
    }

    private static string FormatShortcut(uint modifiers, uint vk)
    {
        var parts = new List<string>();
        if ((modifiers & Interop.HotkeyWin32.MOD_WIN) != 0) parts.Add("Win");
        if ((modifiers & Interop.HotkeyWin32.MOD_ALT) != 0) parts.Add("Alt");
        if ((modifiers & Interop.HotkeyWin32.MOD_CONTROL) != 0) parts.Add("Ctrl");
        if ((modifiers & Interop.HotkeyWin32.MOD_SHIFT) != 0) parts.Add("Shift");
        parts.Add(VkToKeyName(vk));
        return string.Join("+", parts);
    }

    private static string VkToKeyName(uint vk)
    {
        return vk switch
        {
            0x25 => "Left",
            0x26 => "Up",
            0x27 => "Right",
            0x28 => "Down",
            0x0D => "Enter",
            0x2E => "Delete",
            0x31 => "1", 0x32 => "2", 0x33 => "3", 0x34 => "4", 0x35 => "5", 0x36 => "6",
            0x41 => "A", 0x42 => "B", 0x43 => "C", 0x44 => "D", 0x45 => "E", 0x46 => "F",
            0x47 => "G", 0x48 => "H", 0x49 => "I", 0x4A => "J", 0x4B => "K", 0x4E => "N",
            0x50 => "P", 0x51 => "Q", 0x52 => "R", 0x53 => "S", 0x54 => "T", 0x55 => "U",
            0x57 => "W",
            _ => $"0x{vk:X}"
        };
    }
}
