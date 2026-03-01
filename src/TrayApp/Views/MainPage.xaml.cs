namespace TrayApp.Views;

public partial class MainPage : Page
{
    public MainPage()
    {
        InitializeComponent();
        Loaded += OnLoaded;
    }

    private void OnLoaded(object sender, RoutedEventArgs e)
    {
        if (App.Logic is { } logic)
            logic.HotkeyTriggered += OnHotkeyTriggered;
    }

    private void OnHotkeyTriggered(string actionName)
    {
        LastHotkeyText.Text = $"{actionName}  ({DateTime.Now:HH:mm:ss})";
    }
}
