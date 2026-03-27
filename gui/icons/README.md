# Icons Directory

This directory contains the application icons for DB Hub GUI.

## Files

- **appicon.png** - High-resolution source image for application icon (used for generating .icns)
- **appicon.icns** - macOS icon bundle (contains all required sizes: 16x16 to 1024x1024)
- **menubaricon.png** - System menu bar/tray icon (simplified design, optimized for small sizes)
- **icon.png** - Symbolic link to appicon.png (required by Tauri build system)

## Usage

### Application Icon (Finder, Dock, About Window)
The `appicon.icns` file is automatically copied to the app bundle during build:
```
DBHub.app/Contents/Resources/appicon.icns
```

### Menu Bar Icon (System Tray)
The `menubaricon.png` is configured in `gui/tauri.conf.json`:
```json
"systemTray": {
  "iconPath": "icons/menubaricon.png",
  "iconAsTemplate": true
}
```

## Building Icons

To regenerate the `.icns` file from the source PNG:
```bash
make icon
# or
bash scripts/create-icon.sh
```

This script:
1. Reads from `gui/icons/appicon.png`
2. Generates multiple sizes (16x16 to 1024x1024)
3. Creates `gui/icons/appicon.icns` using `iconutil`

## Design Specifications

### Application Icon
- **Size**: 1024x1024 (source), scales down to 16x16
- **Style**: Flat, minimalist, Hub & Spoke design
- **Colors**: Blue (#2563EB) on light gray (#F8FAFC)
- **Elements**: Central database cylinder with 6 surrounding connection dots

### Menu Bar Icon
- **Size**: 32x32 (Retina), 16x16 (standard)
- **Style**: Ultra-minimalist
- **Colors**: Solid white (light mode) or gray (dark mode)
- **Elements**: Simplified database cylinder only

## File Naming Convention

- **appicon** - Application/focus icon (main icon for the app)
- **menubaricon** - System menu bar/tray icon
- Avoid generic names like "dbhub" or "icon" for clarity

## Icon Template Mode

The menu bar icon uses `iconAsTemplate: true`, which allows macOS to automatically adjust the icon's appearance based on the system theme (light/dark mode).
