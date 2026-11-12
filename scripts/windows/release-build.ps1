# Windows: build a release engine with no machine-specific paths inside (use this for anything you publish) into bin\native\windows-x64.
# Usage: powershell -File scripts\windows\release-build.ps1
$root = (Resolve-Path "$PSScriptRoot\..\..").Path
$env:RUSTFLAGS = "--remap-path-prefix=$env:USERPROFILE=~ --remap-path-prefix=$root=."
cargo build --release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$out = "$root\bin\native\windows-x64"
New-Item -ItemType Directory $out -Force | Out-Null
$exe = if ($env:CARGO_TARGET_DIR) { "$env:CARGO_TARGET_DIR\release\morc.exe" } else { "$root\target\release\morc.exe" }
Copy-Item $exe "$out\morc.exe" -Force
"built bin\native\windows-x64\morc.exe; personal paths inside: " + [bool](Select-String -Path "$out\morc.exe" -Pattern ([regex]::Escape($env:USERNAME)) -List -ErrorAction SilentlyContinue)