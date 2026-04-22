param(
    [string]$ApiBaseUrl = "http://10.0.2.2:8070",
    [switch]$AndroidOnly,
    [string]$FlutterPath
)

$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "_mobile-tools.ps1")

$flutterExecutable = Resolve-FlutterExecutable -FlutterPath $FlutterPath
if (-not $flutterExecutable) {
    throw "Flutter SDK was not found. Add it to PATH or pass -FlutterPath."
}

$mobileRoot = Join-Path $PSScriptRoot "..\\apps\\mobile"
$resolvedMobileRoot = Resolve-Path $mobileRoot

Push-Location $resolvedMobileRoot
try {
    $platforms = if ($AndroidOnly) { "android" } else { "android,ios" }

    if (-not (Test-Path ".\\android")) {
        Write-Host "Creating Flutter platform scaffolding..."
        & $flutterExecutable create . --platforms $platforms
    }

    Write-Host "Fetching Flutter dependencies..."
    & $flutterExecutable pub get

    Write-Host ""
    Write-Host "Next step:"
    Write-Host "$flutterExecutable run --dart-define=API_BASE_URL=$ApiBaseUrl"
} finally {
    Pop-Location
}
