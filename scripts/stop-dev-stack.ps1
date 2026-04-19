param(
    [string]$ComposeFile = "infra/docker-compose.yml",
    [switch]$RemoveVolumes
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$composePath = Resolve-Path (Join-Path $repoRoot $ComposeFile)

if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    throw "Docker was not found in PATH."
}

$composeArgs = @("compose", "-f", $composePath, "down")
if ($RemoveVolumes) {
    $composeArgs += "--volumes"
}

& docker @composeArgs

Write-Host "Development stack stopped."
