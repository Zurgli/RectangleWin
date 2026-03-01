using Microsoft.UI.Dispatching;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml.Navigation;

namespace TrayApp;

/// <summary>
/// Provides application-specific behavior to supplement the default Application class.
/// </summary>
public partial class App : Application
{
    private Window? _window = Window.Current;

    public static Window? MainWindow => (Current as App)?._window;
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

            _window.ExtendsContentIntoTitleBar = true;
            AppLog.WriteDebug("ExtendsContentIntoTitleBar set");

            // Remove system title bar (and its min/max/close) so only our custom buttons show
            if (_window.AppWindow?.Presenter is OverlappedPresenter overlapped)
            {
                overlapped.SetBorderAndTitleBar(hasBorder: true, hasTitleBar: false);
                AppLog.WriteDebug("SetBorderAndTitleBar(false) - system title bar hidden");
            }

            Logic = new AppLogic(_window.DispatcherQueue);
            Logic.Start();
            AppLog.WriteDebug("HotkeyManager started");

            StartupRegistration.SetLaunchAtStartup(Logic.Config.LaunchOnLogin);

            Logic.HotkeyManager.AddTrayIcon("RectangleWin", Environment.ProcessPath);
            AppLog.WriteDebug("Tray icon added");

            Logic.HotkeyManager.TrayShowWindowRequested += () =>
            {
                var w = _window;
                if (w == null) return;
                w.DispatcherQueue.TryEnqueue(() => w.AppWindow?.Show(true));
            };

            Logic.HotkeyManager.TrayExitRequested += () =>
            {
                _window!.DispatcherQueue.TryEnqueue(() => Exit());
            };

            _window!.Closed += (_, _) =>
            {
                Logic.Stop();
            };

            try
            {
                _window?.AppWindow?.Resize(new Windows.Graphics.SizeInt32(1000, 1400));
            }
            catch { /* ignore */ }

            _window!.Activate();
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
