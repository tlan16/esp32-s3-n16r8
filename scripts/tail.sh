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

espflash monitor
