#!/bin/bash
set -e

# Script to create macOS .icns icon from source PNG
# Usage: ./scripts/create-icon.sh

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ICON_SRC="${PROJECT_ROOT}/gui/icons/appicon.png"
ICONSET="/tmp/dbhub.iconset"
ICNS_OUTPUT="${PROJECT_ROOT}/gui/icons/appicon.icns"

echo "🎨 Creating macOS icon from ${ICON_SRC}..."

# Check if source icon exists
if [ ! -f "${ICON_SRC}" ]; then
    echo "❌ Error: Source icon not found: ${ICON_SRC}"
    exit 1
fi

# Clean and create iconset directory
rm -rf "${ICONSET}"
mkdir -p "${ICONSET}"

# Define required sizes for macOS icon
SIZES=(
    "16:16x16"
    "32:32x32"
    "64:64x64"
    "128:128x128"
    "256:256x256"
    "512:512x512"
    "1024:1024x1024"
)

# Generate icons at different scales
echo "🔨 Generating icon assets..."
for size_info in "${SIZES[@]}"; do
    IFS=':' read -r base_size dimensions <<< "$size_info"
    
    # 1x scale
    sips -z ${base_size} ${base_size} "${ICON_SRC}" --out "${ICONSET}/icon_${base_size}x${base_size}.png" >/dev/null 2>&1
    
    # 2x scale (Retina)
    retina_size=$((base_size * 2))
    sips -z ${retina_size} ${retina_size} "${ICON_SRC}" --out "${ICONSET}/icon_${base_size}x${base_size}@2x.png" >/dev/null 2>&1
done

# Create .icns file
echo "📦 Creating .icns file..."
iconutil -c icns "${ICONSET}" -o "${ICNS_OUTPUT}"

# Cleanup
rm -rf "${ICONSET}"

echo "✅ Icon created: ${ICNS_OUTPUT}"
ls -lh "${ICNS_OUTPUT}"
