#!/bin/bash
set -e

echo "=== Building Web Label Printer Flatpak ==="

cd "$(dirname "$0")"

# Check for required tools
command -v flatpak-builder >/dev/null 2>&1 || { echo "Error: flatpak-builder not found. Install with: sudo dnf install flatpak-builder"; exit 1; }

# Add Flathub repo and install SDK if needed
echo "Checking for required runtimes..."
flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo 2>/dev/null || true
flatpak install -y flathub org.gnome.Platform//48 org.gnome.Sdk//48 2>/dev/null || true
flatpak install -y flathub org.freedesktop.Sdk.Extension.rust-stable//24.08 2>/dev/null || true
flatpak install -y flathub org.freedesktop.Sdk.Extension.node22//24.08 2>/dev/null || true

# Build the Flatpak
echo "Building Flatpak..."
flatpak-builder --user --install-deps-from=flathub --force-clean --install build-dir com.phomemo.weblabelprinter.yaml

echo ""
echo "=== Build Complete ==="
echo "Run with: flatpak run com.phomemo.weblabelprinter"
echo ""
echo "To create a redistributable .flatpak file:"
echo "  flatpak build-bundle ~/.local/share/flatpak/repo web-label-printer.flatpak com.phomemo.weblabelprinter"
