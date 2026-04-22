function Resolve-FlutterExecutable {
    param(
        [string]$FlutterPath
    )

    $candidates = @()
    if (-not [string]::IsNullOrWhiteSpace($FlutterPath)) {
        $candidates += $FlutterPath
    }
    if ($env:FLUTTER_BIN) {
        $candidates += $env:FLUTTER_BIN
    }
    if ($env:FLUTTER_ROOT) {
        $candidates += $env:FLUTTER_ROOT
    }

    $command = Get-Command flutter -ErrorAction SilentlyContinue
    if ($command) {
        return $command.Source
    }

    foreach ($candidate in $candidates) {
        $resolved = Resolve-FlutterCandidate -Candidate $candidate
        if ($resolved) {
            return $resolved
        }
    }

    return $null
}

function Resolve-FlutterCandidate {
    param(
        [string]$Candidate
    )

    if ([string]::IsNullOrWhiteSpace($Candidate)) {
        return $null
    }

    $normalized = $Candidate.Trim('"')
    if (-not (Test-Path $normalized)) {
        return $null
    }

    $item = Get-Item $normalized
    if (-not $item.PSIsContainer) {
        if ($item.Name -in @("flutter.bat", "flutter")) {
            return $item.FullName
        }
        return $null
    }

    $binBat = Join-Path $item.FullName "bin\flutter.bat"
    if (Test-Path $binBat) {
        return $binBat
    }

    $binSh = Join-Path $item.FullName "bin\flutter"
    if (Test-Path $binSh) {
        return $binSh
    }

    $current = $item.FullName
    for ($i = 0; $i -lt 12; $i++) {
        $parent = Split-Path $current -Parent
        if ([string]::IsNullOrWhiteSpace($parent) -or $parent -eq $current) {
            break
        }

        $parentBinBat = Join-Path $parent "bin\flutter.bat"
        if (Test-Path $parentBinBat) {
            return $parentBinBat
        }

        $parentBinSh = Join-Path $parent "bin\flutter"
        if (Test-Path $parentBinSh) {
            return $parentBinSh
        }
        $current = $parent
    }

    return $null
}

function Resolve-FlutterRoot {
    param(
        [string]$FlutterExecutable
    )

    if ([string]::IsNullOrWhiteSpace($FlutterExecutable)) {
        return $null
    }

    $flutterFile = Get-Item $FlutterExecutable -ErrorAction SilentlyContinue
    if (-not $flutterFile) {
        return $null
    }

    return Split-Path (Split-Path $flutterFile.FullName -Parent) -Parent
}

function Resolve-AndroidSdkRoot {
    param(
        [string]$AndroidSdkRoot
    )

    if (-not [string]::IsNullOrWhiteSpace($AndroidSdkRoot) -and (Test-Path $AndroidSdkRoot)) {
        return (Get-Item $AndroidSdkRoot).FullName
    }
    if ($env:ANDROID_SDK_ROOT -and (Test-Path $env:ANDROID_SDK_ROOT)) {
        return (Get-Item $env:ANDROID_SDK_ROOT).FullName
    }
    if ($env:ANDROID_HOME -and (Test-Path $env:ANDROID_HOME)) {
        return (Get-Item $env:ANDROID_HOME).FullName
    }
    return $null
}

function Resolve-AdbExecutable {
    param(
        [string]$AndroidSdkRoot
    )

    $command = Get-Command adb -ErrorAction SilentlyContinue
    if ($command) {
        return $command.Source
    }

    $sdkRoot = Resolve-AndroidSdkRoot -AndroidSdkRoot $AndroidSdkRoot
    if (-not $sdkRoot) {
        return $null
    }

    $candidate = Join-Path $sdkRoot "platform-tools\adb.exe"
    if (Test-Path $candidate) {
        return $candidate
    }

    return $null
}

function Resolve-EmulatorExecutable {
    param(
        [string]$AndroidSdkRoot
    )

    $command = Get-Command emulator -ErrorAction SilentlyContinue
    if ($command) {
        return $command.Source
    }

    $sdkRoot = Resolve-AndroidSdkRoot -AndroidSdkRoot $AndroidSdkRoot
    if (-not $sdkRoot) {
        return $null
    }

    $candidate = Join-Path $sdkRoot "emulator\emulator.exe"
    if (Test-Path $candidate) {
        return $candidate
    }

    return $null
}
