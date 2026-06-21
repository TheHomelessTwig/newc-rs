#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

echo "Building newc (release)..."
cargo build --release

echo "Installing to /usr/local/bin/..."
sudo cp target/release/newc /usr/local/bin/newc
sudo chmod +x /usr/local/bin/newc

if [[ "$(uname)" == "Linux" ]]; then
    echo "Installing desktop launcher entry..."
    mkdir -p "$HOME/.local/share/applications" "$HOME/.local/share/icons/hicolor/scalable/apps"
    cp packaging/newc.desktop "$HOME/.local/share/applications/newc.desktop"
    cp packaging/newc.svg "$HOME/.local/share/icons/hicolor/scalable/apps/newc.svg"
    command -v update-desktop-database >/dev/null && update-desktop-database "$HOME/.local/share/applications" || true
    command -v gtk-update-icon-cache >/dev/null && gtk-update-icon-cache -f "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
fi

echo "newc installed: $(newc --version)"
