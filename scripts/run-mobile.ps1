param(
    [string]$ApiBaseUrl = "http://10.0.2.2:8070",
    [switch]$StartStack,
    [switch]$StartEmulator,
    [string]$AvdName
)

$ErrorActionPreference = "Stop"
if (Test-Path variable:PSNativeCommandUseErrorActionPreference) {
    $PSNativeCommandUseErrorActionPreference = $false
}

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$mobileRoot = Resolve-Path (Join-Path $repoRoot "apps\mobile")
$startStackScript = Join-Path $repoRoot "scripts\start-dev-stack.ps1"
$bootstrapScript = Join-Path $repoRoot "scripts\bootstrap-mobile.ps1"
$startEmulatorScript = Join-Path $repoRoot "scripts\start-android-emulator.ps1"

function Assert-CommandAvailable {
    param([string]$CommandName)

    if (-not (Get-Command $CommandName -ErrorAction SilentlyContinue)) {
        throw "Required command was not found in PATH: $CommandName"
    }
}

if ($StartStack) {
    & $startStackScript
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to start the backend stack."
    }
}

Assert-CommandAvailable -CommandName "flutter"

Push-Location $mobileRoot
try {
    if (-not (Test-Path ".\android")) {
        Write-Host "Android platform files are missing. Running bootstrap first..."
        & $bootstrapScript -ApiBaseUrl $ApiBaseUrl -AndroidOnly
        if ($LASTEXITCODE -ne 0) {
            throw "Mobile bootstrap failed."
        }
    }

    if ($StartEmulator) {
        if ($AvdName) {
            & $startEmulatorScript -AvdName $AvdName
        } else {
            & $startEmulatorScript
        }
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to start the Android emulator."
        }
    }

    Write-Host "Running Flutter app against $ApiBaseUrl"
    & flutter run "--dart-define=API_BASE_URL=$ApiBaseUrl"
    if ($LASTEXITCODE -ne 0) {
        throw "flutter run failed."
    }
} finally {
    Pop-Location
}
