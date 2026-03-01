using System.Diagnostics;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Navigation;

namespace TrayApp;

/// <summary>
/// Provides application-specific behavior to supplement the default Application class.
/// </summary>
public partial class App : Application
{
    /// <summary>Custom entry point to catch bootstrap exceptions (DISABLE_XAML_GENERATED_MAIN).</summary>
    [STAThread]
    public static int Main(string[] args)
    {
        string logDir = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "RectangleWin");
        string startupLog = Path.Combine(logDir, "startup-log.txt");
        void WriteStartup(string msg)
        {
            try
            {
                Directory.CreateDirectory(logDir);
                File.AppendAllText(startupLog, $"{DateTime.Now:yyyy-MM-dd HH:mm:ss} {msg}{Environment.NewLine}");
            }
            catch { /* ignore */ }
        }

        try
        {
            WriteStartup("Main() entered");
            WinRT.ComWrappersSupport.InitializeComWrappers();
            WriteStartup("ComWrappers initialized");
            Application.Start(_ =>
            {
                WriteStartup("Application.Start callback");
                var ctx = new DispatcherQueueSynchronizationContext(DispatcherQueue.GetForCurrentThread());
                SynchronizationContext.SetSynchronizationContext(ctx);
                new App();
                WriteStartup("App created");
            });
            WriteStartup("Application.Start returned");
            return 0;
        }
        catch (Exception ex)
        {
            string full = ex.ToString();
            WriteStartup("EXCEPTION: " + full);
            try
            {
                _ = Interop.User32Menu.MessageBoxW(nint.Zero, full, "RectangleWin startup failed", Interop.User32Menu.MB_OK | Interop.User32Menu.MB_ICONERROR);
            }
            catch { }
            return ex.HResult;
        }
    }

    private Window? _window = Window.Current;

    public static Window? MainWindow => (Current as App)?._window;
    public static AppLogic? Logic { get; private set; }

    public App()
    {
        AppLog.Write("App constructor start");
        InitializeComponent();
        AppLog.Write("App constructor after Init");
        AppDomain.CurrentDomain.UnhandledException += (_, args) =>
        {
            if (args.ExceptionObject is Exception ex)
            {
                AppLog.Write("UnhandledException: " + ex.Message);
                AppLog.Write(ex);
                try
                {
                    _ = Interop.User32Menu.MessageBoxW(nint.Zero, ex.ToString(), "RectangleWin Error", Interop.User32Menu.MB_OK | Interop.User32Menu.MB_ICONERROR);
                }
                catch { /* ignore */ }
            }
        };
    }

    protected override void OnLaunched(LaunchActivatedEventArgs e)
    {
        AppLog.Write("OnLaunched start");
        try
        {
            AppLog.DebugEnabled = AppConfig.Load().DebugLogging;
            AppLog.Write("OnLaunched config loaded");
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

            try
            {
                StartupRegistration.SetLaunchAtStartup(Logic.Config.LaunchOnLogin);
                Logic.HotkeyManager.AddTrayIcon("RectangleWin", Environment.ProcessPath);
                AppLog.WriteDebug("Tray icon added");
            }
            catch (Exception ex)
            {
                AppLog.Write("Tray/startup setup failed: " + ex.Message);
                AppLog.Write(ex);
            }

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
