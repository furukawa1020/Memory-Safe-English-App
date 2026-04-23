function Resolve-RepoRoot {
    $candidates = @()
    if ($env:MEMORY_SAFE_ENGLISH_ROOT) {
        $candidates += $env:MEMORY_SAFE_ENGLISH_ROOT
    }
    try {
        $candidates += (Get-Location).Path
    } catch {
    }
    $candidates += (Split-Path $PSScriptRoot -Parent)

    foreach ($candidate in $candidates) {
        if ([string]::IsNullOrWhiteSpace($candidate)) {
            continue
        }

        $current = $candidate
        for ($i = 0; $i -lt 6; $i++) {
            if (
                (Test-Path (Join-Path $current "scripts")) -and
                (Test-Path (Join-Path $current "apps")) -and
                (Test-Path (Join-Path $current "services"))
            ) {
                return (Get-Item $current).FullName
            }

            $parent = Split-Path $current -Parent
            if ([string]::IsNullOrWhiteSpace($parent) -or $parent -eq $current) {
                break
            }
            $current = $parent
        }
    }

    return (Split-Path $PSScriptRoot -Parent)
}

function Get-MobileConfigPath {
    $repoRoot = Resolve-RepoRoot
    return Join-Path $repoRoot ".mobile-local.json"
}

function Read-MobileConfig {
    $configPath = Get-MobileConfigPath
    if (-not (Test-Path $configPath)) {
        return @{}
    }

    try {
        $raw = Get-Content $configPath -Raw -Encoding UTF8
        $parsed = $raw | ConvertFrom-Json
        if ($parsed) {
            $config = @{}
            foreach ($property in $parsed.PSObject.Properties) {
                $config[$property.Name] = [string]$property.Value
            }
            return $config
        }
    } catch {
        return @{}
    }

    return @{}
}

function Get-MobileConfigValue {
    param(
        [string]$Key
    )

    $config = Read-MobileConfig
    if ($config.ContainsKey($Key) -and -not [string]::IsNullOrWhiteSpace([string]$config[$Key])) {
        return [string]$config[$Key]
    }
    return $null
}

function Resolve-FlutterExecutable {
    param(
        [string]$FlutterPath
    )

    $candidates = @()
    if (-not [string]::IsNullOrWhiteSpace($FlutterPath)) {
        $candidates += $FlutterPath
    }
    $configFlutterPath = Get-MobileConfigValue -Key "flutter_path"
    if ($configFlutterPath) {
        $candidates += $configFlutterPath
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

function Get-DefaultAndroidSdkRoots {
    $candidates = New-Object System.Collections.Generic.List[string]

    if ($env:LOCALAPPDATA) {
        $candidates.Add((Join-Path $env:LOCALAPPDATA "Android\Sdk"))
    }
    if ($env:USERPROFILE) {
        $candidates.Add((Join-Path $env:USERPROFILE "AppData\Local\Android\Sdk"))
        $candidates.Add((Join-Path $env:USERPROFILE "Android\Sdk"))
    }

    $resolved = New-Object System.Collections.Generic.List[string]
    foreach ($candidate in $candidates) {
        try {
            if (Test-Path $candidate) {
                $resolved.Add($candidate)
            }
        } catch {
            continue
        }
    }

    return @($resolved | Select-Object -Unique)
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
    $configAndroidSdkRoot = Get-MobileConfigValue -Key "android_sdk_root"
    if ($configAndroidSdkRoot -and (Test-Path $configAndroidSdkRoot)) {
        return (Get-Item $configAndroidSdkRoot).FullName
    }
    if ($env:ANDROID_SDK_ROOT -and (Test-Path $env:ANDROID_SDK_ROOT)) {
        return (Get-Item $env:ANDROID_SDK_ROOT).FullName
    }
    if ($env:ANDROID_HOME -and (Test-Path $env:ANDROID_HOME)) {
        return (Get-Item $env:ANDROID_HOME).FullName
    }
    foreach ($candidate in Get-DefaultAndroidSdkRoots) {
        return (Get-Item $candidate).FullName
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

    $legacyCandidate = Join-Path $sdkRoot "tools\emulator.exe"
    if (Test-Path $legacyCandidate) {
        return $legacyCandidate
    }

    return $null
}

function Invoke-FlutterCommand {
    param(
        [string]$FlutterExecutable,
        [string[]]$Arguments,
        [string]$AndroidSdkRoot
    )

    if ([string]::IsNullOrWhiteSpace($FlutterExecutable)) {
        throw "Flutter executable is required."
    }

    $resolvedAndroidSdkRoot = Resolve-AndroidSdkRoot -AndroidSdkRoot $AndroidSdkRoot

    if ($resolvedAndroidSdkRoot) {
        $env:ANDROID_SDK_ROOT = $resolvedAndroidSdkRoot
        if (-not $env:ANDROID_HOME) {
            $env:ANDROID_HOME = $resolvedAndroidSdkRoot
        }
    }

    if (-not $env:CI) {
        $env:CI = "true"
    }

    $finalArguments = @("--suppress-analytics") + $Arguments
    & $FlutterExecutable @finalArguments
}
