; RectangleWin Inno Setup script
; Build: Publish TrayApp for win-x64 (Release), then run iscc RectangleWin.iss from this folder.

#define MyAppName "RectangleWin"
#define MyAppExeName "TrayApp.exe"
#define MyAppVersion "0.1"
#define PublishDir "..\src\TrayApp\bin\Release\net8.0-windows10.0.19041.0\win-x64\publish"

[Setup]
AppId={{A1B2C3D4-E5F6-7890-ABCD-EF1234567890}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher=zurgli
AppPublisherURL=https://github.com/zurgli
AppSupportURL=https://github.com/zurgli
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
OutputDir=.
OutputBaseFilename=RectangleWin-Setup-{#MyAppVersion}
Compression=lz2
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=lowest
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64

[Tasks]
Name: "launchatstartup"; Description: "Launch RectangleWin when Windows starts"; GroupDescription: "Startup:"; Flags: unchecked

[Files]
Source: "{#PublishDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Registry]
Root: HKCU; Subkey: "SOFTWARE\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "RectangleWin"; ValueData: """{app}\{#MyAppExeName}"""; Flags: uninsdeletevalue; Tasks: launchatstartup

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\Uninstall {#MyAppName}"; Filename: "{uninstallexe}"

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "Launch {#MyAppName}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
Type: dirifempty; Name: "{app}"
