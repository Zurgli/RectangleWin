# RectangleWin installer

Inno Setup script for a single-file installer with optional "Launch at startup".

## Prerequisites

- [Inno Setup 6](https://jrsoftware.org/isinfo.php) (or use `winget install JRSoftware.InnoSetup`)
- TrayApp published for win-x64 (Release)

## Build installer

1. Publish the app from the repo root:
   ```bash
   dotnet publish src\TrayApp\TrayApp.csproj -c Release -r win-x64 --self-contained true
   ```
   Or use Visual Studio: right-click TrayApp → Publish → win-x64 profile.

2. From this folder, compile the installer:
   ```bash
   "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" RectangleWin.iss
   ```
   Or open `RectangleWin.iss` in Inno Setup and Build → Compile.

3. Output: `RectangleWin-Setup-0.1.exe` in this folder.

## Installer options (finish page)

- **Launch RectangleWin when Windows starts** – adds RectangleWin to the current user’s Run key (same as the in-app "Launch at startup" setting).
- **Launch RectangleWin** – runs the app when the installer finishes.
