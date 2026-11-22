# build.ps1 - Cross-platform kernel and ISO build script for RustOS
$ErrorActionPreference = "Stop"
$root = (Resolve-Path "$PSScriptRoot\..").Path
Set-Location $root

Write-Host "==> Compiling RustOS kernel (i686-rustos)..."
$cargo = "$env:USERPROFILE\.cargo\bin\cargo.exe"
if (-not (Test-Path $cargo)) {
    $cmd = Get-Command "cargo" -ErrorAction SilentlyContinue
    if ($cmd) { $cargo = $cmd.Source }
    else { throw "cargo.exe not found." }
}

& $cargo build --bin rustos --target i686-rustos.json -Z build-std=core,compiler_builtins,alloc
if ($LASTEXITCODE -ne 0) { throw "Cargo build failed" }

if (-not (Test-Path "iso\boot\grub")) {
    New-Item -ItemType Directory -Path "iso\boot\grub" -Force | Out-Null
}
if (-not (Test-Path "build")) {
    New-Item -ItemType Directory -Path "build" -Force | Out-Null
}

Copy-Item "target\i686-rustos\debug\rustos" -Destination "iso\boot\kernel.elf" -Force

Write-Host "==> Generating bootable ISO using grub-mkrescue..."
# First check if WSL is available
$wsl = Get-Command "wsl.exe" -ErrorAction SilentlyContinue
if ($wsl) {
    try {
        $wslPath = (wsl.exe -d Ubuntu-22.04 wslpath -u ($root -replace '\\', '/')).Trim()
        wsl.exe -d Ubuntu-22.04 bash "$wslPath/scripts/build_iso.sh"
        if ($LASTEXITCODE -eq 0) {
            Write-Host "==> ISO build succeeded: build\rustos.iso"
            return
        }
    } catch {
        Write-Warning "WSL build attempt encountered an issue: $_"
    }
}

# Fallback: Docker
$docker = Get-Command "docker" -ErrorAction SilentlyContinue
if ($docker) {
    Write-Host "Attempting build via Docker container..."
    docker build -t rustos-builder .
    docker run --rm -v "${root}:/work" rustos-builder
    if ($LASTEXITCODE -eq 0) {
        Write-Host "==> Docker build succeeded: build\rustos.iso"
        return
    }
}

throw "Neither WSL with Ubuntu-22.04 nor Docker was able to run grub-mkrescue. Please ensure WSL2 Ubuntu or Docker is installed."
