# Windows portable archive: dist/release/keyprobe-<version>-windows-x64-portable.zip
# Runs on Windows with Rust (MSVC).
$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..\..')

$target = 'x86_64-pc-windows-msvc'
# The one place of the version: [workspace.package] version in Cargo.toml (the first `version =`).
$version = (Select-String -Path Cargo.toml -Pattern '^version = "(.+)"' | Select-Object -First 1).Matches[0].Groups[1].Value

cargo build --release --locked -p keyprobe --target $target
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$name = "keyprobe-$version-windows-x64-portable"
$stage = "dist/stage/$name"
if (Test-Path $stage) { Remove-Item -Recurse -Force $stage }
New-Item -ItemType Directory -Force "$stage/licenses", 'dist/release' | Out-Null
Copy-Item "target/$target/release/keyprobe.exe" "$stage/KeyProbe.exe"
Copy-Item LICENSE-MIT "$stage/LICENSE-MIT.txt"
Copy-Item LICENSE-APACHE "$stage/LICENSE-APACHE.txt"
Copy-Item assets/fonts/UFL.txt, assets/fonts/Hack-Regular.txt "$stage/licenses/"
Compress-Archive -Path $stage -DestinationPath "dist/release/$name.zip" -Force
Write-Host "done: dist/release/$name.zip"
