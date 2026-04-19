param(
    [string]$ComposeFile = "infra/docker-compose.yml",
    [int]$StartupTimeoutSeconds = 240,
    [switch]$NoBuild,
    [switch]$SkipSmokeTest,
    [string]$ProxyBaseUrl = "http://127.0.0.1:8070",
    [string]$AdminToken = "dev-proxy-admin-token"
)

$ErrorActionPreference = "Stop"
if (Test-Path variable:PSNativeCommandUseErrorActionPreference) {
    $PSNativeCommandUseErrorActionPreference = $false
}

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$composePath = Resolve-Path (Join-Path $repoRoot $ComposeFile)
$smokeTestPath = Join-Path $repoRoot "scripts\smoke-test.ps1"
$targetContainers = @(
    "mse-postgres",
    "mse-redis",
    "mse-worker",
    "mse-api",
    "mse-proxy"
)

function Assert-CommandAvailable {
    param([string]$CommandName)

    if (-not (Get-Command $CommandName -ErrorAction SilentlyContinue)) {
        throw "Required command was not found in PATH: $CommandName"
    }
}

function Get-ContainerHealth {
    param([string]$ContainerName)

    $output = & docker inspect --format "{{if .State.Health}}{{.State.Health.Status}}{{else}}{{.State.Status}}{{end}}" $ContainerName 2>$null
    if ($LASTEXITCODE -ne 0) {
        return "missing"
    }

    return $output.Trim()
}

function Wait-ForHealthyContainers {
    param(
        [string[]]$ContainerNames,
        [int]$TimeoutSeconds
    )

    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)

    while ((Get-Date) -lt $deadline) {
        $statuses = @{}
        foreach ($containerName in $ContainerNames) {
            $statuses[$containerName] = Get-ContainerHealth -ContainerName $containerName
        }

        $allHealthy = $true
        foreach ($status in $statuses.Values) {
            if ($status -ne "healthy") {
                $allHealthy = $false
                break
            }
        }

        if ($allHealthy) {
            Write-Host "All containers are healthy."
            return
        }

        Write-Host "Waiting for containers to become healthy..."
        foreach ($entry in $statuses.GetEnumerator()) {
            Write-Host ("- {0}: {1}" -f $entry.Key, $entry.Value)
        }

        Start-Sleep -Seconds 3
    }

    Write-Host ""
    Write-Host "Timed out while waiting for healthy containers. Current compose status:"
    & docker compose -f $composePath ps
    throw "Development stack did not become healthy within $TimeoutSeconds seconds."
}

Assert-CommandAvailable -CommandName "docker"

Write-Host "Starting local development stack..."
$composeArgs = @("compose", "-f", $composePath, "up", "-d")
if (-not $NoBuild) {
    $composeArgs += "--build"
}
& docker @composeArgs
if ($LASTEXITCODE -ne 0) {
    throw "docker compose up failed."
}

Wait-ForHealthyContainers -ContainerNames $targetContainers -TimeoutSeconds $StartupTimeoutSeconds

if (-not $SkipSmokeTest) {
    Write-Host ""
    Write-Host "Running smoke test..."
    & $smokeTestPath -ProxyBaseUrl $ProxyBaseUrl -AdminToken $AdminToken
    if ($LASTEXITCODE -ne 0) {
        throw "Smoke test failed."
    }
}

Write-Host ""
Write-Host "Development stack is ready."
