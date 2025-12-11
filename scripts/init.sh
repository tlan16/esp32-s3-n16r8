#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.." || exit 1

echo "Working directory: $(pwd)"

echo "Installing espup..."
cargo install espup --locked
espup install --esp-riscv-gcc --extended-llvm
. "$HOME/export-esp.sh"

echo "Installing espflash..."
cargo install espflash --locked
espflash --version

echo "Installing espmonitor..."
cargo install espmonitor

echo "Installing probe-rs-tools..."
cargo install probe-rs-tools

printf "\nDone\n"
