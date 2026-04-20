param(
    [string]$ProxyBaseUrl = "http://127.0.0.1:8070"
)

$ErrorActionPreference = "Stop"
if (Test-Path variable:PSNativeCommandUseErrorActionPreference) {
    $PSNativeCommandUseErrorActionPreference = $false
}

function Test-CommandAvailable {
    param([string]$CommandName)
    [bool](Get-Command $CommandName -ErrorAction SilentlyContinue)
}

function Get-CheckResult {
    param(
        [string]$Name,
        [bool]$Passed,
        [string]$Details
    )

    [pscustomobject]@{
        name    = $Name
        passed  = $Passed
        details = $Details
    }
}

function Get-AndroidSdkRoot {
    if ($env:ANDROID_SDK_ROOT) {
        return $env:ANDROID_SDK_ROOT
    }
    if ($env:ANDROID_HOME) {
        return $env:ANDROID_HOME
    }
    return $null
}

function Get-EmulatorExecutable {
    $command = Get-Command emulator -ErrorAction SilentlyContinue
    if ($command) {
        return $command.Source
    }

    $sdkRoot = Get-AndroidSdkRoot
    if (-not $sdkRoot) {
        return $null
    }

    $candidate = Join-Path $sdkRoot "emulator\emulator.exe"
    if (Test-Path $candidate) {
        return $candidate
    }

    return $null
}

$results = New-Object System.Collections.Generic.List[object]

$dockerInstalled = Test-CommandAvailable -CommandName "docker"
$dockerCliDetails = if ($dockerInstalled) { "Docker CLI found in PATH." } else { "Install Docker Desktop or Docker CLI." }
$results.Add((Get-CheckResult -Name "docker_cli" -Passed $dockerInstalled -Details $dockerCliDetails))

$dockerDaemonReady = $false
if ($dockerInstalled) {
    cmd /c "docker info >nul 2>nul"
    $dockerDaemonReady = ($LASTEXITCODE -eq 0)
}
$dockerDaemonDetails = if ($dockerDaemonReady) { "Docker daemon is reachable." } else { "Start Docker Desktop or the Docker service." }
$results.Add((Get-CheckResult -Name "docker_daemon" -Passed $dockerDaemonReady -Details $dockerDaemonDetails))

$flutterInstalled = Test-CommandAvailable -CommandName "flutter"
$flutterDetails = if ($flutterInstalled) { "Flutter SDK found in PATH." } else { "Install Flutter and add it to PATH." }
$results.Add((Get-CheckResult -Name "flutter_sdk" -Passed $flutterInstalled -Details $flutterDetails))

$adbInstalled = Test-CommandAvailable -CommandName "adb"
$adbDetails = if ($adbInstalled) { "Android Debug Bridge found in PATH." } else { "Install Android platform tools or Android Studio." }
$results.Add((Get-CheckResult -Name "adb" -Passed $adbInstalled -Details $adbDetails))

$emulatorExecutable = Get-EmulatorExecutable
$emulatorInstalled = -not [string]::IsNullOrWhiteSpace($emulatorExecutable)
$emulatorDetails = if ($emulatorInstalled) { "Android emulator command was found." } else { "Install Android emulator tools or Android Studio." }
$results.Add((Get-CheckResult -Name "android_emulator" -Passed $emulatorInstalled -Details $emulatorDetails))

$avdAvailable = $false
$avdDetails = "No Android Virtual Device found."
if ($emulatorInstalled) {
    try {
        $avdList = @(& $emulatorExecutable -list-avds | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
        if ($avdList.Count -gt 0) {
            $avdAvailable = $true
            $avdDetails = "Available AVDs: $($avdList -join ', ')"
        } else {
            $avdDetails = "Android emulator is installed, but no AVD is configured."
        }
    } catch {
        $avdDetails = "Failed to query AVDs. Check Android SDK setup."
    }
}
$results.Add((Get-CheckResult -Name "android_avd" -Passed $avdAvailable -Details $avdDetails))

$proxyReady = $false
try {
    $response = Invoke-WebRequest -Uri "$ProxyBaseUrl/ready" -Method GET -TimeoutSec 3
    $proxyReady = ($response.StatusCode -eq 200)
} catch {
    $proxyReady = $false
}
$proxyDetails = if ($proxyReady) { "Proxy is ready at $ProxyBaseUrl." } else { "Proxy is not ready. Start the stack with scripts/start-dev-stack.ps1." }
$results.Add((Get-CheckResult -Name "proxy_ready" -Passed $proxyReady -Details $proxyDetails))

foreach ($result in $results) {
    $marker = if ($result.passed) { "[OK]" } else { "[WARN]" }
    Write-Host ("{0} {1}: {2}" -f $marker, $result.name, $result.details)
}

$failedChecks = @($results | Where-Object { -not $_.passed })
if ($failedChecks.Count -gt 0) {
    Write-Host ""
    Write-Host "Suggested next steps:"

    if ($failedChecks.name -contains "docker_daemon") {
        Write-Host "- Start Docker Desktop, then run .\scripts\start-dev-stack.ps1"
    }
    if ($failedChecks.name -contains "flutter_sdk") {
        Write-Host "- Install Flutter, then run .\scripts\bootstrap-mobile.ps1"
    }
    if ($failedChecks.name -contains "adb") {
        Write-Host "- Install Android Studio or platform tools and start an emulator"
    }
    if ($failedChecks.name -contains "android_emulator") {
        Write-Host "- Install Android emulator tools or Android Studio"
    }
    if ($failedChecks.name -contains "android_avd") {
        Write-Host "- Create an Android Virtual Device, then run .\scripts\start-android-emulator.ps1"
    }
    if ($failedChecks.name -contains "proxy_ready" -and $dockerDaemonReady) {
        Write-Host "- Start the backend stack, then retry the doctor script"
    }

    exit 1
}

Write-Host ""
Write-Host "All development prerequisites are ready."
