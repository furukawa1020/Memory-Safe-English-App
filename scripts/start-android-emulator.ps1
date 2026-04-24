param(
    [string]$AvdName,
    [int]$BootTimeoutSeconds = 180,
    [string]$AndroidSdkRoot,
    [switch]$WipeData
)

$ErrorActionPreference = "Stop"
if (Test-Path variable:PSNativeCommandUseErrorActionPreference) {
    $PSNativeCommandUseErrorActionPreference = $false
}
. (Join-Path $PSScriptRoot "_mobile-tools.ps1")

function Get-AvailableAvds {
    param([string]$EmulatorPath)

    $output = & $EmulatorPath -list-avds
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to list Android Virtual Devices."
    }

    return @($output | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
}

function Wait-ForEmulatorBoot {
    param(
        [string]$AdbPath,
        [int]$TimeoutSeconds
    )

    & $AdbPath wait-for-device
    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while ((Get-Date) -lt $deadline) {
        $bootOutput = & $AdbPath shell getprop sys.boot_completed 2>$null
        $bootCompleted = if ($null -eq $bootOutput) {
            ""
        } else {
            ($bootOutput | Out-String).Trim()
        }
        if ($bootCompleted -eq "1") {
            return
        }
        Start-Sleep -Seconds 3
    }

    throw "Timed out while waiting for the Android emulator to finish booting."
}

$emulatorPath = Resolve-EmulatorExecutable -AndroidSdkRoot $AndroidSdkRoot
if (-not $emulatorPath) {
    throw "Android emulator command was not found. Install Android Studio or Android SDK emulator tools first."
}

$adbPath = Resolve-AdbExecutable -AndroidSdkRoot $AndroidSdkRoot
if (-not $adbPath) {
    throw "adb was not found. Install Android platform tools first."
}

$availableAvds = Get-AvailableAvds -EmulatorPath $emulatorPath
if (-not $availableAvds -or $availableAvds.Count -eq 0) {
    throw "No Android Virtual Device was found. Create an AVD in Android Studio first."
}

$selectedAvd = $AvdName
if (-not $selectedAvd) {
    $selectedAvd = Get-MobileConfigValue -Key "avd_name"
}
if (-not $selectedAvd) {
    if ($availableAvds.Count -eq 1) {
        $selectedAvd = $availableAvds[0]
    } else {
        $selectedAvd = $availableAvds[0]
        Write-Host "Multiple AVDs were found. Using the first one by default: $selectedAvd"
        Write-Host "Available AVDs:"
        $availableAvds | ForEach-Object { Write-Host "- $_" }
    }
}

if ($availableAvds -notcontains $selectedAvd) {
    throw "AVD '$selectedAvd' was not found. Available AVDs: $($availableAvds -join ', ')"
}

Write-Host "Starting Android emulator: $selectedAvd"
$arguments = @("-avd", $selectedAvd)
if ($WipeData) {
    $arguments += "-wipe-data"
}
Start-Process -FilePath $emulatorPath -ArgumentList $arguments | Out-Null

Write-Host "Waiting for emulator boot to complete..."
Wait-ForEmulatorBoot -AdbPath $adbPath -TimeoutSeconds $BootTimeoutSeconds

Write-Host "Android emulator is ready."
