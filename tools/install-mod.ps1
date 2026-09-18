param(
    [Parameter(Mandatory)]
    [string]$GameDirectory,
    [string]$Configuration = 'Release'
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$binary = Join-Path $root "build\$Configuration\mod.shroudedit.dll"
if (-not (Test-Path -LiteralPath $binary)) { throw "Build output is missing: $binary" }
if (-not (Test-Path -LiteralPath (Join-Path $GameDirectory 'enshrouded.exe'))) { throw "GameDirectory must contain enshrouded.exe." }

$destination = Join-Path $GameDirectory 'mods\mod.shroudedit'
New-Item -ItemType Directory -Force -Path $destination | Out-Null
Copy-Item -LiteralPath $binary,(Join-Path $root 'mod.json') -Destination $destination -Force
Write-Host "Installed ShroudEdit to $destination"
