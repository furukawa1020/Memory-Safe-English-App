param(
    [string]$ComposeFile = "infra/docker-compose.yml",
    [switch]$RemoveVolumes
)

$ErrorActionPreference = "Stop"
if (Test-Path variable:PSNativeCommandUseErrorActionPreference) {
    $PSNativeCommandUseErrorActionPreference = $false
}

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$composePath = Resolve-Path (Join-Path $repoRoot $ComposeFile)

if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    throw "Docker was not found in PATH."
}
& docker info *> $null
if ($LASTEXITCODE -ne 0) {
    throw "Docker daemon is not reachable. Start Docker Desktop or the Docker service, then retry."
}

$composeArgs = @("compose", "-f", $composePath, "down")
if ($RemoveVolumes) {
    $composeArgs += "--volumes"
}

& docker @composeArgs
if ($LASTEXITCODE -ne 0) {
    throw "docker compose down failed."
}

Write-Host "Development stack stopped."
