param(
    [string]$ApiBaseUrl = "http://10.0.2.2:8070",
    [switch]$StartStack,
    [switch]$StartEmulator,
    [string]$AvdName,
    [string]$FlutterPath,
    [string]$AndroidSdkRoot,
    [switch]$SkipPubGet,
    [switch]$Detached,
    [string]$DeviceId,
    [string]$ApplicationId = "com.example.memory_safe_english_mobile",
    [string]$ActivityName = ".MainActivity",
    [switch]$WipeEmulatorData
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

function Get-ConnectedAndroidDeviceId {
    param(
        [string]$AdbExecutable
    )

    $deviceLines = & $AdbExecutable devices 2>$null | Select-Object -Skip 1
    foreach ($line in $deviceLines) {
        if ($line -match "^\s*(\S+)\s+device\s*$") {
            return $Matches[1]
        }
    }

    return $null
}

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

$adbExecutable = Resolve-AdbExecutable -AndroidSdkRoot $AndroidSdkRoot
if (-not $adbExecutable) {
    throw "adb was not found. Pass -AndroidSdkRoot or configure it in .mobile-local.json."
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
        $startEmulatorParams = @{
            AndroidSdkRoot = $AndroidSdkRoot
        }
        if ($AvdName) {
            $startEmulatorParams.AvdName = $AvdName
        }
        if ($WipeEmulatorData) {
            $startEmulatorParams.WipeData = $true
        }
        & $startEmulatorScript @startEmulatorParams
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to start the Android emulator."
        }
    }

    $resolvedDeviceId = $DeviceId
    if (-not $resolvedDeviceId) {
        $resolvedDeviceId = Get-ConnectedAndroidDeviceId -AdbExecutable $adbExecutable
    }

    if ($Detached) {
        if (-not $resolvedDeviceId) {
            throw "No connected Android device was found for detached launch."
        }

        Write-Host "Building debug APK for detached install..."
        $buildArguments = @("build", "apk", "--debug", "--dart-define=API_BASE_URL=$ApiBaseUrl")
        if ($resolvedDeviceId) {
            $buildArguments += @("-d", $resolvedDeviceId)
        }
        Invoke-FlutterCommand -FlutterExecutable $flutterExecutable -AndroidSdkRoot $AndroidSdkRoot -Arguments $buildArguments
        if ($LASTEXITCODE -ne 0) {
            throw "flutter build apk failed."
        }

        $apkPath = Join-Path $mobileRoot "build\app\outputs\flutter-apk\app-debug.apk"
        if (-not (Test-Path $apkPath)) {
            throw "Debug APK was not found at $apkPath."
        }

        Write-Host "Installing debug APK on $resolvedDeviceId"
        & $adbExecutable -s $resolvedDeviceId install -r $apkPath
        if ($LASTEXITCODE -ne 0) {
            throw "adb install failed."
        }

        $componentName = "$ApplicationId/$ActivityName"
        Write-Host "Launching $componentName on $resolvedDeviceId"
        & $adbExecutable -s $resolvedDeviceId shell am start -n $componentName
        if ($LASTEXITCODE -ne 0) {
            throw "adb launch failed."
        }
    } else {
        Write-Host "Running Flutter app against $ApiBaseUrl"
        $runArguments = @("run", "--dart-define=API_BASE_URL=$ApiBaseUrl")
        if ($resolvedDeviceId) {
            $runArguments += @("-d", $resolvedDeviceId)
        }
        Invoke-FlutterCommand -FlutterExecutable $flutterExecutable -AndroidSdkRoot $AndroidSdkRoot -Arguments $runArguments
        if ($LASTEXITCODE -ne 0) {
            throw "flutter run failed."
        }
    }
} finally {
    Pop-Location
}
