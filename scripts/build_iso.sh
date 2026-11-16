#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"
mkdir -p build
mkdir -p iso/boot/grub

grub-mkrescue -o build/rustos.iso iso
echo "Successfully built build/rustos.iso"
