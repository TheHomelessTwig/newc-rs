#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

echo "Building newc (release)..."
cargo build --release

echo "Installing to /usr/local/bin/..."
sudo cp target/release/newc /usr/local/bin/newc
sudo chmod +x /usr/local/bin/newc

echo "newc installed: $(newc --version)"
