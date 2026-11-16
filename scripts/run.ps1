# run.ps1 - VirtualBox runner script for RustOS
param(
    [switch]$Headless,
    [switch]$Rebuild
)

$ErrorActionPreference = "Stop"
$root = (Resolve-Path "$PSScriptRoot\..").Path
Set-Location $root

if ($Rebuild -or (-not (Test-Path "build\rustos.iso"))) {
    & "$PSScriptRoot\build.ps1"
}

# Locate VBoxManage
$vbox = "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe"
if (-not (Test-Path $vbox)) {
    $cmd = Get-Command "VBoxManage" -ErrorAction SilentlyContinue
    if ($cmd) { $vbox = $cmd.Source }
    else { throw "VBoxManage.exe not found at standard path or on PATH." }
}

$vmName = "RustOS"
$vms = & $vbox list vms
$existing = $vms | Select-String "`"$vmName`""

if (-not $existing) {
    Write-Host "Creating VirtualBox VM '$vmName'..."
    & $vbox createvm --name $vmName --ostype Other --register
    & $vbox modifyvm $vmName --memory 128 --boot1 dvd --firmware bios
    & $vbox modifyvm $vmName --uart1 0x3F8 4 --uart-mode1 file "$root\build\serial.log"
    & $vbox storagectl $vmName --name "SATA" --add sata --bootable on
    & $vbox storageattach $vmName --storagectl "SATA" --port 0 --device 0 --type dvddrive --medium "$root\build\rustos.iso"
} else {
    # Refresh settings
    & $vbox modifyvm $vmName --uart1 0x3F8 4 --uart-mode1 file "$root\build\serial.log"
    & $vbox storageattach $vmName --storagectl "SATA" --port 0 --device 0 --type dvddrive --medium "$root\build\rustos.iso"
}

if ($Headless) {
    Write-Host "Starting $vmName (headless)..."
    & $vbox startvm $vmName --type headless
} else {
    Write-Host "Starting $vmName (GUI)..."
    & $vbox startvm $vmName --type gui
}
