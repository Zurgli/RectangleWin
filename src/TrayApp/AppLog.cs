namespace TrayApp;

internal static class AppLog
{
    private static readonly string LogPath = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
        "RectangleWin", "log.txt");
    private static readonly object Lock = new();

    /// <summary>When true, WriteDebug outputs to log. Set from config at startup.</summary>
    public static bool DebugEnabled { get; set; }

    public static void Write(string message)
    {
        lock (Lock)
        {
            try
            {
                Directory.CreateDirectory(Path.GetDirectoryName(LogPath)!);
                string line = $"{DateTime.Now:yyyy-MM-dd HH:mm:ss} {message}{Environment.NewLine}";
                File.AppendAllText(LogPath, line);
            }
            catch { /* ignore */ }
        }
    }

    public static void WriteDebug(string message)
    {
        if (!DebugEnabled) return;
        Write(message);
    }

    public static void Write(Exception ex)
    {
        Write($"ERROR: {ex.Message}");
        Write(ex.StackTrace ?? "");
        if (ex.InnerException is { } inner)
            Write(inner);
    }
}
