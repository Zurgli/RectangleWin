[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"

$stagedFiles = @(git diff --cached --name-only --diff-filter=ACMR)
if ($LASTEXITCODE -ne 0) {
    throw "Unable to read staged files."
}

if ($stagedFiles.Count -eq 0) {
    Write-Host "pre-commit: no staged files"
    exit 0
}

$runFrontend = $false
$runBackend = $false

foreach ($file in $stagedFiles) {
    if (
        $file -match '^app/src/' -or
        $file -match '^app/index\.html$' -or
        $file -match '^app/package(-lock)?\.json$' -or
        $file -match '^app/tsconfig(\.node)?\.json$' -or
        $file -match '^app/vite\.config\.ts$' -or
        $file -match '^app/src-tauri/tauri\.conf\.json$'
    ) {
        $runFrontend = $true
    }

    if ($file -match '^app/src-tauri/') {
        $runBackend = $true
    }
}

if (-not $runFrontend -and -not $runBackend) {
    Write-Host "pre-commit: no app checks required"
    exit 0
}

$checkParams = @{}
if ($runFrontend) {
    $checkParams.Frontend = $true
}
if ($runBackend) {
    $checkParams.Backend = $true
}

& (Join-Path $PSScriptRoot "check.ps1") @checkParams
