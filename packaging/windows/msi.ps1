# Windows installer: dist/release/keyprobe-<version>-windows-x64.msi
# Run after package.ps1 (it uses the folder package.ps1 prepared). Needs WiX Toolset v5:
#   dotnet tool install --global wix --version 5.0.2
$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..\..')

$version = (Select-String -Path Cargo.toml -Pattern '^version = "(.+)"' | Select-Object -First 1).Matches[0].Groups[1].Value
# MSI takes numeric versions only: 0.2.0-rc.1 -> 0.2.0.
$msiVersion = $version -replace '-.*$', ''

$stage = "dist/stage/keyprobe-$version-windows-x64-portable"
if (-not (Test-Path "$stage/KeyProbe.exe")) {
    throw "${stage} is missing: run packaging/windows/package.ps1 first"
}
$out = "dist/release/keyprobe-$version-windows-x64.msi"
wix build packaging/windows/keyprobe.wxs -arch x64 -pdbtype none `
    -d "Version=$msiVersion" -d "Stage=$((Resolve-Path $stage).Path)" `
    -d "Icon=$((Resolve-Path 'assets/keyprobe.ico').Path)" -o $out
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Write-Host "done: $out"
