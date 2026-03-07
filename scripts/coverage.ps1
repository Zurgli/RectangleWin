[CmdletBinding()]
param(
    [switch]$Html,
    [switch]$Lcov,
    [switch]$InstallPrereqs,
    [string]$PreferredInstallPath = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools"
)

$ErrorActionPreference = "Stop"

if ($Html.IsPresent -and $Lcov.IsPresent) {
    throw "Choose either -Html or -Lcov, not both."
}

if (-not $Html.IsPresent -and -not $Lcov.IsPresent) {
    $Html = $true
}

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
. (Join-Path $PSScriptRoot "enter-vsdevshell.ps1") -PreferredInstallPath $PreferredInstallPath

function Test-LlvmToolsInstalled {
    $installedComponents = rustup component list --installed
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to query installed rustup components."
    }

    return $installedComponents | Where-Object { $_ -like "llvm-tools-*" } | Select-Object -First 1
}

function Ensure-CargoLlvmCov {
    $null = cargo llvm-cov --version 2>$null
    return $LASTEXITCODE -eq 0
}

if (-not (Test-LlvmToolsInstalled)) {
    if ($InstallPrereqs.IsPresent) {
        rustup component add llvm-tools-preview
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to install rustup component llvm-tools-preview."
        }
    } else {
        throw "Missing rustup component llvm-tools-preview. Run `rustup component add llvm-tools-preview` or rerun this script with -InstallPrereqs."
    }
}

if (-not (Ensure-CargoLlvmCov)) {
    if ($InstallPrereqs.IsPresent) {
        cargo install cargo-llvm-cov
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to install cargo-llvm-cov."
        }
    } else {
        throw "Missing cargo-llvm-cov. Run `cargo install cargo-llvm-cov` or rerun this script with -InstallPrereqs."
    }
}

Push-Location (Join-Path $repoRoot "app\src-tauri")
try {
    if ($Html.IsPresent) {
        $outputDir = "target/llvm-cov-html"
        cargo llvm-cov --html --output-dir $outputDir
        if ($LASTEXITCODE -ne 0) {
            throw "cargo llvm-cov failed."
        }
        Write-Host "Rust coverage HTML report written to app/src-tauri/$outputDir/html/index.html"
    } else {
        $outputPath = "target/llvm-cov.info"
        cargo llvm-cov --lcov --output-path $outputPath
        if ($LASTEXITCODE -ne 0) {
            throw "cargo llvm-cov failed."
        }
        Write-Host "Rust coverage LCOV report written to app/src-tauri/$outputPath"
    }
}
finally {
    Pop-Location
}
