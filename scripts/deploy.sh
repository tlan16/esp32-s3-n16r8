#!/usr/bin/env bash

set -euo pipefail

cd "$(dirname "$0")/.." || exit 1

echo "Working directory: $(pwd)"

esp_init_file="$HOME/export-esp.sh"

if [ ! -f "$esp_init_file" ]; then
    echo "ESP environment file not found at $esp_init_file. Please run the scripts/init.sh script first."
    exit 1
fi

# shellcheck disable=SC1090
. "$esp_init_file"

port="$(espflash list-ports | grep --only-matching --extended-regexp '^/dev/cu\S+')"

if [ -z "$port" ]; then
    echo "No ESP device found. Please connect your device and try again."
    echo "Available ports:"
    espflash list-ports
    exit 1
fi

echo "port=$port"

printf "\nBoard Information:\n"
echo "====================="
espflash board-info --port "$port"

printf "\n"

printf "Build rust code:\n"
echo "====================="
cargo build --release

printf "\n"

printf "Flashing device:\n"
echo "====================="

# For esp-hal with no_std, binary is in xtensa-esp32s3-none-elf
built_binary="target/xtensa-esp32s3-none-elf/release/esp32-s3-n16r8"
echo "built_binary=$built_binary"

if [ ! -f "$built_binary" ]; then
    echo "Error: Binary not found at $built_binary"
    exit 1
fi

espflash flash --port "$port" "$built_binary"

printf "\n✓ Done\n"
