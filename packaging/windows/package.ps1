<#
.SYNOPSIS
  Build and package MineRust for Windows.

.DESCRIPTION
  Produces, in $env:DIST (default: dist/release):
    minerust-<version>-windows-<arch>-portable.zip   portable bundle (minerust.exe + assets/)

  The binary links the C runtime statically (+crt-static), eliminating the need
  for the Visual C++ redistributable.
#>
param(
  [ValidateSet('x64', 'x86', 'arm64')] [string] $Arch = 'x64',
  [switch] $SkipBuild
)
$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path

function Invoke-Native([string] $What, [scriptblock] $Block) {
  Write-Output "==> $What"
  & $Block
  if ($LASTEXITCODE -ne 0) { throw "$What failed with exit code $LASTEXITCODE" }
}

# Read version from Cargo.toml
$Version = $env:MINERUST_VERSION
if (-not $Version) {
  $inPkg = $false
  foreach ($line in Get-Content (Join-Path $Root 'Cargo.toml')) {
    if ($line -match '^\s*\[package\]') { $inPkg = $true; continue }
    if ($line -match '^\s*\[' -and $line -notmatch '^\s*\[package\]') { $inPkg = $false; continue }
    if ($inPkg -and $line -match '^\s*version\s*=\s*"([^"]+)"') { $Version = $Matches[1]; break }
  }
}
if (-not $Version) { throw 'could not read [package] version from Cargo.toml' }

$Target = switch ($Arch) {
  'x64'   { 'x86_64-pc-windows-msvc' }
  'x86'   { 'i686-pc-windows-msvc' }
  'arm64' { 'aarch64-pc-windows-msvc' }
}
$Dist = if ($env:DIST) { $env:DIST } else { Join-Path $Root 'dist\release' }
$TargetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $Root 'target' }
New-Item -ItemType Directory -Force -Path $Dist | Out-Null

Write-Output "MineRust $Version for Windows $Arch ($Target)"

if (-not $SkipBuild) {
  # Static CRT: no VC++ redistributable needed
  $flagVar = 'CARGO_TARGET_' + ($Target.ToUpper() -replace '-', '_') + '_RUSTFLAGS'
  [Environment]::SetEnvironmentVariable($flagVar, '-C target-feature=+crt-static')
  Invoke-Native "cargo build ($Target)" { cargo build --release --locked --target $Target }
}

$Bin = Join-Path $TargetDir "$Target\release"
if (-not (Test-Path (Join-Path $Bin 'minerust.exe'))) {
  $Bin = Join-Path $TargetDir "release"
}

# ---- portable zip ------------------------------------------------------------------------------
$Portable = Join-Path $TargetDir "windows-package\minerust-$Version-windows-$Arch-portable"
Remove-Item -Recurse -Force $Portable -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $Portable | Out-Null

Copy-Item (Join-Path $Bin 'minerust.exe') $Portable
Copy-Item -Recurse (Join-Path $Root 'assets') (Join-Path $Portable 'assets')
New-Item -ItemType Directory -Force -Path (Join-Path $Portable 'saves') | Out-Null
foreach ($f in 'README.md', 'LICENSE', 'LICENSE-MIT', 'LICENSE-APACHE') {
  $p = Join-Path $Root $f
  if (Test-Path $p) { Copy-Item $p $Portable }
}

$Zip = Join-Path $Dist "minerust-$Version-windows-$Arch-portable.zip"
Remove-Item -Force $Zip -ErrorAction SilentlyContinue
Compress-Archive -Path $Portable -DestinationPath $Zip
Write-Output "wrote $Zip"

Get-Item $Zip | Format-Table Name, Length
