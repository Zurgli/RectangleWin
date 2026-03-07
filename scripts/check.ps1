[CmdletBinding()]
param(
    [switch]$Frontend,
    [switch]$Backend
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$runFrontend = $Frontend.IsPresent
$runBackend = $Backend.IsPresent

if (-not $runFrontend -and -not $runBackend) {
    $runFrontend = $true
    $runBackend = $true
}

if ($runFrontend) {
    Write-Host "Running frontend build check..."
    Push-Location (Join-Path $repoRoot "app")
    try {
        npm run build
    }
    finally {
        Pop-Location
    }
}

if ($runBackend) {
    Write-Host "Running Rust tests..."
    . (Join-Path $PSScriptRoot "enter-vsdevshell.ps1")
    Push-Location (Join-Path $repoRoot "app\src-tauri")
    try {
        cargo test
    }
    finally {
        Pop-Location
    }
}
