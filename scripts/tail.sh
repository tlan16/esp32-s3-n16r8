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

echo "Attaching to ESP32-S3 via probe-rs for RTT logs..."
echo "This will show defmt logs from rtt_target"
echo "Press Ctrl+C to exit"
echo ""

# Path to the binary (release build)
binary_path="target/xtensa-esp32s3-none-elf/release/esp32-s3-n16r8"

if [ ! -f "$binary_path" ]; then
    echo "Error: Binary not found at $binary_path"
    echo "Please build the project first with: cargo build --release"
    exit 1
fi

killall probe-rs
probe-rs attach --chip esp32s3 --rtt-scan-memory "$binary_path"
