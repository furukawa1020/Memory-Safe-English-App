param(
    [string]$AndroidSdkRoot,
    [string]$SystemImagePackage = "system-images;android-36.1;google_apis_playstore;x86_64",
    [switch]$InstallEmulatorPackage = $true
)

$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "_mobile-tools.ps1")

$resolvedAndroidSdkRoot = Resolve-AndroidSdkRoot -AndroidSdkRoot $AndroidSdkRoot
if (-not $resolvedAndroidSdkRoot) {
    throw "Android SDK root could not be resolved."
}

Write-Host "Using Android SDK root: $resolvedAndroidSdkRoot"

Write-Host ""
Write-Host "Accepting Android SDK licenses..."
Invoke-AndroidSdkManager -AndroidSdkRoot $resolvedAndroidSdkRoot -Arguments @("--licenses") -AutoAcceptLicenses

$packages = New-Object System.Collections.Generic.List[string]
if ($InstallEmulatorPackage) {
    $packages.Add("emulator")
}
if (-not [string]::IsNullOrWhiteSpace($SystemImagePackage)) {
    $packages.Add($SystemImagePackage)
}

if ($packages.Count -gt 0) {
    Write-Host ""
    Write-Host "Installing packages..."
    Invoke-AndroidSdkManager -AndroidSdkRoot $resolvedAndroidSdkRoot -Arguments @($packages.ToArray()) -AutoAcceptLicenses
}

Write-Host ""
Write-Host "Android emulator dependencies are prepared."
