param(
    [string]$ApiBaseUrl = "http://10.0.2.2:8070",
    [switch]$AndroidOnly
)

$ErrorActionPreference = "Stop"

$flutter = Get-Command flutter -ErrorAction SilentlyContinue
if (-not $flutter) {
    throw "Flutter SDK was not found in PATH. Install Flutter first, then rerun this script."
}

$mobileRoot = Join-Path $PSScriptRoot "..\\apps\\mobile"
$resolvedMobileRoot = Resolve-Path $mobileRoot

Push-Location $resolvedMobileRoot
try {
    $platforms = if ($AndroidOnly) { "android" } else { "android,ios" }

    if (-not (Test-Path ".\\android")) {
        Write-Host "Creating Flutter platform scaffolding..."
        flutter create . --platforms $platforms
    }

    Write-Host "Fetching Flutter dependencies..."
    flutter pub get

    Write-Host ""
    Write-Host "Next step:"
    Write-Host "flutter run --dart-define=API_BASE_URL=$ApiBaseUrl"
} finally {
    Pop-Location
}
