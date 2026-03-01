using Microsoft.UI.Dispatching;
using Microsoft.UI.Xaml.Navigation;

namespace TrayApp;

/// <summary>
/// Provides application-specific behavior to supplement the default Application class.
/// </summary>
public partial class App : Application
{
    private Window? _window = Window.Current;

    public static AppLogic? Logic { get; private set; }

    public App()
    {
        InitializeComponent();
        AppDomain.CurrentDomain.UnhandledException += (_, args) =>
        {
            if (args.ExceptionObject is Exception ex)
            {
                AppLog.Write("UnhandledException: " + ex.Message);
                AppLog.Write(ex);
            }
        };
    }

    protected override void OnLaunched(LaunchActivatedEventArgs e)
    {
        try
        {
            AppLog.DebugEnabled = AppConfig.Load().DebugLogging;
            AppLog.WriteDebug("OnLaunched start");
            _window ??= new Window();
            AppLog.WriteDebug("Window created");

            if (_window.Content is not Frame rootFrame)
            {
                rootFrame = new Frame();
                rootFrame.NavigationFailed += OnNavigationFailed;
                _window.Content = rootFrame;
            }
            AppLog.WriteDebug("Frame ready");

            _ = rootFrame.Navigate(typeof(MainPage), e.Arguments);
            AppLog.WriteDebug("MainPage navigated");

            Logic = new AppLogic(_window.DispatcherQueue);
            Logic.Start();
            AppLog.WriteDebug("HotkeyManager started");

            StartupRegistration.SetLaunchAtStartup(Logic.Config.LaunchOnLogin);

            Logic.HotkeyManager.AddTrayIcon("RectangleWin");
            AppLog.WriteDebug("Tray icon added");

            Logic.HotkeyManager.TrayExitRequested += () =>
            {
                _window!.DispatcherQueue.TryEnqueue(() => Exit());
            };

            _window.Closed += (_, _) =>
            {
                Logic.Stop();
            };

            _window.Activate();
            AppLog.WriteDebug("OnLaunched complete");
        }
        catch (Exception ex)
        {
            AppLog.Write("OnLaunched failed");
            AppLog.Write(ex);
            _ = Interop.User32Menu.MessageBoxW(
                nint.Zero,
                ex.ToString(),
                "RectangleWin Error",
                Interop.User32Menu.MB_OK | Interop.User32Menu.MB_ICONERROR);
            throw;
        }
    }

    void OnNavigationFailed(object sender, NavigationFailedEventArgs e)
    {
        throw new Exception("Failed to load Page " + e.SourcePageType.FullName);
    }
}
