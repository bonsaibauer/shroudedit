param(
    [string]$ShroudtopiaRepository = 'A:\Github\shroudtopia'
)

$ErrorActionPreference = 'Stop'
$loaderOutput = Join-Path $ShroudtopiaRepository 'build\x64'
$modOutput = Join-Path (Split-Path -Parent $PSScriptRoot) 'build\Release\mod.shroudedit.dll'
if (-not (Test-Path -LiteralPath (Join-Path $loaderOutput 'shroudtopia.dll'))) { throw 'Build Shroudtopia first.' }
if (-not (Test-Path -LiteralPath $modOutput)) { throw 'Build ShroudEdit first.' }

$harnessRoot = Join-Path (Split-Path -Parent $PSScriptRoot) 'build\loader-integration-harness'
if (Test-Path -LiteralPath $harnessRoot) { Remove-Item -LiteralPath $harnessRoot -Recurse -Force }
New-Item -ItemType Directory -Force -Path $harnessRoot | Out-Null
Copy-Item -LiteralPath (Join-Path $loaderOutput 'shroudtopia.dll'),(Join-Path $loaderOutput 'winmm.dll') -Destination $harnessRoot -Force
$modsRoot = Join-Path $harnessRoot 'mods'
Get-ChildItem -LiteralPath (Join-Path $ShroudtopiaRepository 'mods') -Directory | ForEach-Object {
    $manifestPath = Join-Path $_.FullName 'mod.json'
    if (-not (Test-Path -LiteralPath $manifestPath)) { return }
    $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
    $target = Join-Path $modsRoot ([string]$manifest.id)
    New-Item -ItemType Directory -Force -Path $target | Out-Null
    Copy-Item -LiteralPath $manifestPath,(Join-Path $loaderOutput ([string]$manifest.shroudtopia.binary)) -Destination $target -Force
}

$package = Join-Path $harnessRoot 'mods\mod.shroudedit'
New-Item -ItemType Directory -Force -Path $package | Out-Null
Copy-Item -LiteralPath $modOutput -Destination $package -Force
Copy-Item -LiteralPath (Join-Path (Split-Path -Parent $PSScriptRoot) 'mod.json') -Destination $package -Force
$clientHarness = Join-Path $harnessRoot 'enshrouded.exe'
Copy-Item -LiteralPath (Join-Path $loaderOutput 'platform-api-smoke.exe') -Destination $clientHarness -Force

try {
    Push-Location $harnessRoot
    & $clientHarness 2>&1 | Tee-Object -Variable loaderOutputLines
    $exitCode = $LASTEXITCODE
    $outputText = $loaderOutputLines -join "`n"
    if ($exitCode -ne 0) { throw "Shroudtopia integration harness failed with exit code $exitCode." }
    if ($outputText -match '\[shroudtopia\]\[ERROR\]') {
        throw 'Shroudtopia reported an integration error.'
    }
    if ($outputText -notmatch 'Registered mod: mod\.shroudedit v0\.2\.0') {
        throw 'ShroudEdit was not registered with its canonical mod ID.'
    }
    exit 0
}
finally {
    Pop-Location
    Remove-Item -LiteralPath $harnessRoot -Recurse -Force -ErrorAction SilentlyContinue
}

