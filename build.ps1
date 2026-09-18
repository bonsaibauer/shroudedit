param(
    [string]$BuildNumber = $(if ($env:SHROUDEDIT_BUILD_NUMBER) { $env:SHROUDEDIT_BUILD_NUMBER } else { 'dev' }),
    [string]$ShroudtopiaDirectory = (Join-Path (Split-Path -Parent $PSScriptRoot) 'shroudtopia'),
    [string]$RefName,
    [ValidateSet('branch', 'tag')]
    [string]$RefType = 'branch'
)

$ErrorActionPreference = 'Stop'
$root = $PSScriptRoot
$version = (Get-Content -LiteralPath (Join-Path $root 'VERSION') -Raw).Trim()
if ($version -notmatch '^\d+\.\d+\.\d+$') { throw "VERSION must use MAJOR.MINOR.PATCH: $version" }
if ($BuildNumber -notmatch '^[A-Za-z0-9._-]+$') { throw "Invalid build number: $BuildNumber" }
if ($RefName) {
    $expected = if ($RefType -eq 'tag') { "v$version" } else { $version }
    if ($RefName -ne $expected) { throw "Version mismatch: $RefType '$RefName', expected '$expected'." }
}
if (-not (Test-Path -LiteralPath (Join-Path $ShroudtopiaDirectory 'api\shroudtopia.h'))) { throw "Shroudtopia API not found: $ShroudtopiaDirectory" }

$buildDirectory = Join-Path $root 'build'
& cmake -S $root -B $buildDirectory -A x64 "-DSHROUDTOPIA_API_DIR=$ShroudtopiaDirectory"
if ($LASTEXITCODE -ne 0) { throw "CMake configure failed with exit code $LASTEXITCODE." }
& cmake --build $buildDirectory --config Release --parallel
if ($LASTEXITCODE -ne 0) { throw "CMake build failed with exit code $LASTEXITCODE." }
& ctest --test-dir $buildDirectory -C Release --output-on-failure
if ($LASTEXITCODE -ne 0) { throw "Tests failed with exit code $LASTEXITCODE." }

$binary = Join-Path $buildDirectory 'Release\mod.shroudedit.dll'
if (-not (Test-Path -LiteralPath $binary)) { throw "Mod binary missing: $binary" }
$package = Join-Path $buildDirectory 'package'
$archive = Join-Path $buildDirectory "shroudedit-$version-$BuildNumber.zip"
$checksum = "$archive.sha256"
Remove-Item -LiteralPath $package -Recurse -Force -ErrorAction SilentlyContinue
Get-ChildItem -LiteralPath $buildDirectory -Filter "shroudedit-$version-*.zip*" -File -ErrorAction SilentlyContinue | Remove-Item -Force
$modDirectory = Join-Path $package 'mods\mod.shroudedit'
New-Item -ItemType Directory -Force -Path $modDirectory | Out-Null
Copy-Item -LiteralPath $binary,(Join-Path $root 'mod.json'),(Join-Path $root 'README.md'),(Join-Path $root 'LICENSE'),(Join-Path $root 'VALIDATED-BUILD.md') -Destination $modDirectory
Compress-Archive -Path "$package\*" -DestinationPath $archive
Remove-Item -LiteralPath $package -Recurse -Force
$hash = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
"$hash  $([IO.Path]::GetFileName($archive))" | Set-Content -LiteralPath $checksum -Encoding ascii
Write-Host "ShroudEdit $version-$BuildNumber built, tested and packaged: $archive"
Write-Host "SHA-256: $hash"
