param(
    [string]$FlutterPath,
    [string]$AndroidSdkRoot,
    [string]$AvdName
)

$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "_mobile-tools.ps1")

$configPath = Get-MobileConfigPath
$current = Read-MobileConfig

if (-not [string]::IsNullOrWhiteSpace($FlutterPath)) {
    $current["flutter_path"] = $FlutterPath
}
if (-not [string]::IsNullOrWhiteSpace($AndroidSdkRoot)) {
    $current["android_sdk_root"] = $AndroidSdkRoot
}
if (-not [string]::IsNullOrWhiteSpace($AvdName)) {
    $current["avd_name"] = $AvdName
}

$current | ConvertTo-Json -Depth 5 | Set-Content -Path $configPath -Encoding UTF8

Write-Host "Saved mobile config to $configPath"
Get-Content $configPath -Encoding UTF8
