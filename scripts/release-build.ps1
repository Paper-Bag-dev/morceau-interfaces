# Build a release exe with no machine-specific paths inside (use this for anything you publish).
# Usage: powershell -File scripts\release-build.ps1
$root = (Resolve-Path "$PSScriptRoot\..").Path
$env:RUSTFLAGS = "--remap-path-prefix=$env:USERPROFILE=~ --remap-path-prefix=$root=."
cargo build --release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
New-Item -ItemType Directory "$root\bin" -Force | Out-Null
$exe = if ($env:CARGO_TARGET_DIR) { "$env:CARGO_TARGET_DIR\release\morc.exe" } else { "$root\target\release\morc.exe" }
Copy-Item $exe "$root\bin\morc.exe" -Force
"built bin\morc.exe; personal paths inside: " + [bool](Select-String -Path "$root\bin\morc.exe" -Pattern ([regex]::Escape($env:USERNAME)) -List -ErrorAction SilentlyContinue)