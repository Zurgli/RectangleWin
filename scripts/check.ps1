[CmdletBinding()]
param(
    [switch]$Frontend,
    [switch]$Backend,
    [ValidateSet("fast", "full")]
    [string]$Mode = "full"
)

$ErrorActionPreference = "Stop"

function Invoke-NativeOrThrow {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Command,
        [string[]]$Arguments = @()
    )

    & $Command @Arguments
    if ($LASTEXITCODE -ne 0) {
        $argText = if ($Arguments.Count -gt 0) {
            " $($Arguments -join ' ')"
        } else {
            ""
        }
        throw "$Command$argText failed with exit code $LASTEXITCODE"
    }
}

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$runFrontend = $Frontend.IsPresent
$runBackend = $Backend.IsPresent

if (-not $runFrontend -and -not $runBackend) {
    $runFrontend = $true
    $runBackend = $true
}

if ($runFrontend) {
    if ($Mode -eq "fast") {
        Write-Host "Running frontend typecheck..."
    } else {
        Write-Host "Running frontend build check..."
    }
    Push-Location (Join-Path $repoRoot "app")
    try {
        if ($Mode -eq "fast") {
            Invoke-NativeOrThrow "npm" @("run", "typecheck")
        } else {
            Invoke-NativeOrThrow "npm" @("run", "build")
        }
        Invoke-NativeOrThrow "npm" @("run", "test")
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
        Invoke-NativeOrThrow "cargo" @("test")
    }
    finally {
        Pop-Location
    }
}
