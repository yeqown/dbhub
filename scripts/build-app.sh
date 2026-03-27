#!/bin/bash
set -e

APP_NAME="DBHub"
APP_BUNDLE="${APP_NAME}.app"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BINARY="${PROJECT_ROOT}/target/release/dbhub-gui"
BUILD_DIR="${PROJECT_ROOT}/gui"

cd "${BUILD_DIR}"

echo "📦 Creating .app bundle..."

# Clean old build
rm -rf "${APP_BUNDLE}"

# Create directory structure
mkdir -p "${APP_BUNDLE}/Contents/MacOS"
mkdir -p "${APP_BUNDLE}/Contents/Resources"

# Copy binary
echo "📝 Copying binary..."
cp "${BINARY}" "${APP_BUNDLE}/Contents/MacOS/${APP_NAME}"
chmod +x "${APP_BUNDLE}/Contents/MacOS/${APP_NAME}"

# Create Info.plist
echo "📝 Creating Info.plist..."
cat > "${APP_BUNDLE}/Contents/Info.plist" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>DBHub</string>
    <key>CFBundleIdentifier</key>
    <string>com.dbhub.app</string>
    <key>CFBundleName</key>
    <string>DBHub</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.4.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>CFBundleIconFile</key>
    <string>appicon</string>
    <key>LSUIElement</key>
    <true/>
</dict>
</plist>
EOF

# Copy icon
echo "📝 Copying icon..."
if [ -f "icons/appicon.icns" ]; then
    cp icons/appicon.icns "${APP_BUNDLE}/Contents/Resources/"
else
    echo "⚠️  Warning: icons/appicon.icns not found, skipping icon"
fi

# Copy resources
echo "📝 Copying resources..."
cp -R public "${APP_BUNDLE}/Contents/Resources/"

echo ""
echo "✅ .app bundle created: ${BUILD_DIR}/${APP_BUNDLE}"
echo "📊 Size: $(du -sh "${APP_BUNDLE}" | cut -f1)"
echo ""
echo "To install:"
echo "  make install-gui         # Install to ~/Applications"
echo "  sudo cp -R ${APP_BUNDLE} /Applications/  # Install to /Applications"
