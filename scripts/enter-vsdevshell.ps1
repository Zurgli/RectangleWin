[CmdletBinding()]
param(
    [string]$PreferredInstallPath = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools",
    [string]$Arch = "x64"
)

$ErrorActionPreference = "Stop"

function Test-VisualStudioToolchain {
    param([string]$InstallPath)

    if ([string]::IsNullOrWhiteSpace($InstallPath)) {
        return $false
    }

    $devShellModule = Join-Path $InstallPath "Common7\Tools\Microsoft.VisualStudio.DevShell.dll"
    $msvcRoot = Join-Path $InstallPath "VC\Tools\MSVC"
    if (-not (Test-Path $devShellModule) -or -not (Test-Path $msvcRoot)) {
        return $false
    }

    $excptHeader = Get-ChildItem $msvcRoot -Recurse -Filter "excpt.h" -ErrorAction SilentlyContinue |
        Select-Object -First 1
    return $null -ne $excptHeader
}

$installCandidates = New-Object System.Collections.Generic.List[string]
if (-not [string]::IsNullOrWhiteSpace($PreferredInstallPath)) {
    $installCandidates.Add($PreferredInstallPath)
}

$vswhere = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path $vswhere) {
    $detectedPaths = & $vswhere `
        -products * `
        -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 `
        -property installationPath

    if ($LASTEXITCODE -eq 0) {
        foreach ($path in $detectedPaths) {
            if (-not [string]::IsNullOrWhiteSpace($path)) {
                $installCandidates.Add($path.Trim())
            }
        }
    }
}

$resolvedInstallPath = $installCandidates |
    Select-Object -Unique |
    Where-Object { Test-VisualStudioToolchain $_ } |
    Select-Object -First 1

if (-not $resolvedInstallPath) {
    throw @"
No usable Visual Studio C++ toolchain was found.

Install Visual Studio Build Tools 2022 or Visual Studio with:
- Desktop development with C++
- MSVC v143 x64/x86 build tools
- Windows 10/11 SDK
"@
}

$devShellModule = Join-Path $resolvedInstallPath "Common7\Tools\Microsoft.VisualStudio.DevShell.dll"
Import-Module $devShellModule
Enter-VsDevShell -VsInstallPath $resolvedInstallPath -DevCmdArguments "-host_arch=$Arch -arch=$Arch" | Out-Null

$compilerPath = (Get-Command cl).Source
Write-Host "Loaded Visual Studio developer shell from $resolvedInstallPath"
Write-Host "Using compiler: $compilerPath"
