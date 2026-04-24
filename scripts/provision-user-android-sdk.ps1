param(
    [string]$SourceAndroidSdkRoot,
    [string]$DestinationAndroidSdkRoot = ".android-sdk",
    [string]$SystemImagePackage = "system-images;android-36.1;google_apis_playstore;x86_64",
    [switch]$SaveConfig
)

$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "_mobile-tools.ps1")

$repoRoot = Resolve-RepoRoot
$resolvedSourceSdkRoot = Resolve-AndroidSdkRoot -AndroidSdkRoot $SourceAndroidSdkRoot
if (-not $resolvedSourceSdkRoot) {
    throw "Source Android SDK root could not be resolved."
}

$sourceSdkManager = Resolve-SdkManagerExecutable -AndroidSdkRoot $resolvedSourceSdkRoot
if (-not $sourceSdkManager) {
    throw "sdkmanager.bat was not found in the source Android SDK."
}

$resolvedDestinationRoot = $DestinationAndroidSdkRoot
if (-not [System.IO.Path]::IsPathRooted($resolvedDestinationRoot)) {
    $resolvedDestinationRoot = Join-Path $repoRoot $resolvedDestinationRoot
}

New-Item -ItemType Directory -Force -Path $resolvedDestinationRoot | Out-Null

$sourceCmdlineTools = Join-Path $resolvedSourceSdkRoot "cmdline-tools"
$destinationCmdlineTools = Join-Path $resolvedDestinationRoot "cmdline-tools"
if ((Test-Path $sourceCmdlineTools) -and -not (Test-Path $destinationCmdlineTools)) {
    Write-Host "Copying cmdline-tools into writable SDK root..."
    Copy-Item -Path $sourceCmdlineTools -Destination $destinationCmdlineTools -Recurse
}

Write-Host "Using source sdkmanager: $sourceSdkManager"
Write-Host "Provisioning writable SDK root: $resolvedDestinationRoot"

Write-Host ""
Write-Host "Accepting Android SDK licenses in writable SDK root..."
Invoke-AndroidSdkManager `
    -AndroidSdkRoot $resolvedDestinationRoot `
    -SdkManagerPath $sourceSdkManager `
    -Arguments @("--licenses") `
    -AutoAcceptLicenses

Write-Host ""
Write-Host "Installing Android platform tools, emulator, platform, and system image..."
Invoke-AndroidSdkManager `
    -AndroidSdkRoot $resolvedDestinationRoot `
    -SdkManagerPath $sourceSdkManager `
    -Arguments @(
        "platform-tools",
        "emulator",
        "platforms;android-36",
        $SystemImagePackage
    ) `
    -AutoAcceptLicenses

if ($SaveConfig) {
    & (Join-Path $PSScriptRoot "save-mobile-config.ps1") -AndroidSdkRoot $resolvedDestinationRoot
}

Write-Host ""
Write-Host "Writable Android SDK is ready: $resolvedDestinationRoot"
