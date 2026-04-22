param(
    [string]$ApiBaseUrl = "http://10.0.2.2:8070",
    [switch]$StartStack,
    [switch]$StartEmulator,
    [string]$AvdName,
    [string]$FlutterPath,
    [string]$AndroidSdkRoot,
    [switch]$SkipPubGet
)

$ErrorActionPreference = "Stop"
if (Test-Path variable:PSNativeCommandUseErrorActionPreference) {
    $PSNativeCommandUseErrorActionPreference = $false
}
. (Join-Path $PSScriptRoot "_mobile-tools.ps1")

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$mobileRoot = Resolve-Path (Join-Path $repoRoot "apps\mobile")
$startStackScript = Join-Path $repoRoot "scripts\start-dev-stack.ps1"
$bootstrapScript = Join-Path $repoRoot "scripts\bootstrap-mobile.ps1"
$startEmulatorScript = Join-Path $repoRoot "scripts\start-android-emulator.ps1"

if ($StartStack) {
    & $startStackScript
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to start the backend stack."
    }
}

$flutterExecutable = Resolve-FlutterExecutable -FlutterPath $FlutterPath
if (-not $flutterExecutable) {
    throw "Flutter SDK was not found. Add it to PATH or pass -FlutterPath."
}

Push-Location $mobileRoot
try {
    if (-not (Test-Path ".\android")) {
        Write-Host "Android platform files are missing. Running bootstrap first..."
        & $bootstrapScript -ApiBaseUrl $ApiBaseUrl -AndroidOnly -FlutterPath $FlutterPath -AndroidSdkRoot $AndroidSdkRoot -SkipPubGet:$SkipPubGet
        if ($LASTEXITCODE -ne 0) {
            throw "Mobile bootstrap failed."
        }
    }

    if ($StartEmulator) {
        if ($AvdName) {
            & $startEmulatorScript -AvdName $AvdName -AndroidSdkRoot $AndroidSdkRoot
        } else {
            & $startEmulatorScript -AndroidSdkRoot $AndroidSdkRoot
        }
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to start the Android emulator."
        }
    }

    Write-Host "Running Flutter app against $ApiBaseUrl"
    Invoke-FlutterCommand -FlutterExecutable $flutterExecutable -AndroidSdkRoot $AndroidSdkRoot -Arguments @("run", "--dart-define=API_BASE_URL=$ApiBaseUrl")
    if ($LASTEXITCODE -ne 0) {
        throw "flutter run failed."
    }
} finally {
    Pop-Location
}
