#Requires -Version 5.1
<#
.SYNOPSIS
    Build and run CodexBar for Windows.

.DESCRIPTION
    Checks that build prerequisites are installed (Rust, MinGW-w64),
    installs them if missing, then builds and launches CodexBar.

.PARAMETER Release
    Build in release mode (optimised). Default is debug.

.PARAMETER SkipBuild
    Skip the build step and run the last built binary.

.PARAMETER Verbose
    Pass -v to CodexBar for debug logging.

.EXAMPLE
    .\dev.ps1                  # debug build + run
    .\dev.ps1 -Release         # release build + run
    .\dev.ps1 -SkipBuild       # run last build
    .\dev.ps1 -Verbose         # debug build + run with verbose logging
#>

param(
    [switch]$Release,
    [switch]$SkipBuild,
    [switch]$Verbose
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$RepoRoot = $PSScriptRoot
$RustDir = Join-Path $RepoRoot "rust"

# ── Ensure known tool paths are in current session PATH ─────────────────────

$knownPaths = @("$env:USERPROFILE\.cargo\bin", "C:\mingw64\bin")
foreach ($p in $knownPaths) {
    if ((Test-Path $p) -and ($env:PATH -notlike "*$p*")) {
        $env:PATH = "$p;$env:PATH"
    }
}

# ── Check prerequisites ─────────────────────────────────────────────────────

$hasCargo = [bool](Get-Command cargo -ErrorAction SilentlyContinue)
$hasDlltool = [bool](Get-Command dlltool -ErrorAction SilentlyContinue)

if (-not $hasCargo -or -not $hasDlltool) {
    $missing = @()
    if (-not $hasCargo)   { $missing += "cargo (Rust)" }
    if (-not $hasDlltool) { $missing += "dlltool (MinGW-w64)" }
    Write-Host "Missing prerequisites: $($missing -join ', ')" -ForegroundColor Yellow
    Write-Host "Running setup script..." -ForegroundColor Cyan
    Write-Host ""

    $setupScript = Join-Path $RepoRoot "scripts\setup-windows.ps1"
    if (-not (Test-Path $setupScript)) {
        Write-Host "ERROR: Setup script not found at $setupScript" -ForegroundColor Red
        exit 1
    }

    & $setupScript

    # Re-check after setup
    $hasCargo = [bool](Get-Command cargo -ErrorAction SilentlyContinue)
    $hasDlltool = [bool](Get-Command dlltool -ErrorAction SilentlyContinue)
    if (-not $hasCargo -or -not $hasDlltool) {
        Write-Host ""
        Write-Host "ERROR: Prerequisites still missing after setup." -ForegroundColor Red
        Write-Host "Please restart your terminal and try again." -ForegroundColor Yellow
        exit 1
    }
}

# ── Build ────────────────────────────────────────────────────────────────────

if (-not $SkipBuild) {
    Push-Location $RustDir
    try {
        if ($Release) {
            Write-Host "Building CodexBar (release)..." -ForegroundColor Cyan
            cargo build --bin codexbar --release
            if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
        } else {
            Write-Host "Building CodexBar (debug)..." -ForegroundColor Cyan
            cargo build --bin codexbar
            if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
        }
    } finally {
        Pop-Location
    }
}

# ── Run ──────────────────────────────────────────────────────────────────────

if ($Release) {
    $binary = Join-Path $RustDir "target\release\codexbar.exe"
} else {
    $binary = Join-Path $RustDir "target\debug\codexbar.exe"
}

if (-not (Test-Path $binary)) {
    Write-Host "ERROR: Binary not found at $binary" -ForegroundColor Red
    Write-Host "Run without -SkipBuild to build first." -ForegroundColor Yellow
    exit 1
}

$args_ = @("menubar")
if ($Verbose) {
    $args_ = @("-v") + $args_
}

Write-Host ""
Write-Host "Starting CodexBar..." -ForegroundColor Green
& $binary @args_
