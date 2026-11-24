#!/usr/bin/env pwsh
# Build script for the WASM renderer.
# Usage:
#   .\build.ps1            # release build
#   .\build.ps1 -Debug     # debug build
#   .\build.ps1 -Serve     # build then start the dev server

param(
    [switch]$Debug,
    [switch]$Serve
)

$ErrorActionPreference = 'Stop'

$Cargo       = "$env:USERPROFILE\.cargo\bin\cargo.exe"
$WasmBindgen = "$env:USERPROFILE\.cargo\bin\wasm-bindgen.exe"

if (-not (Test-Path $Cargo))       { throw "cargo not found at $Cargo" }
if (-not (Test-Path $WasmBindgen)) { throw "wasm-bindgen not found at $WasmBindgen. Install with: cargo install wasm-bindgen-cli" }

$Profile = if ($Debug) { 'debug' } else { 'release' }
$BuildArgs = @('build', '--lib', '--target', 'wasm32-unknown-unknown')
if (-not $Debug) { $BuildArgs += '--release' }

Write-Host "==> cargo $($BuildArgs -join ' ')" -ForegroundColor Cyan
& $Cargo @BuildArgs
if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }

$WasmIn = ".\target\wasm32-unknown-unknown\$Profile\renderer.wasm"
if (-not (Test-Path $WasmIn)) { throw "expected wasm not found: $WasmIn" }

Write-Host "==> wasm-bindgen --target web --out-dir pkg $WasmIn" -ForegroundColor Cyan
& $WasmBindgen --target web --out-dir pkg $WasmIn
if ($LASTEXITCODE -ne 0) { throw "wasm-bindgen failed" }

Write-Host "==> done. Output in .\pkg\" -ForegroundColor Green

if ($Serve) {
    Write-Host "==> starting server" -ForegroundColor Cyan
    node serve.js
}
