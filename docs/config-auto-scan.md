# Configuration File Auto-Scan Feature

## Overview

DB Hub now automatically scans and loads all configuration files from the `~/.dbhub/` directory, eliminating the need to set the `DBHUB_CONFIG` environment variable for most use cases.

## How It Works

### Priority Order

1. **Environment Variable (Backward Compatibility)**
   - If `DBHUB_CONFIG` is set, it takes priority
   - Format: `export DBHUB_CONFIG=~/.dbhub/config.yml:~/.dbhub/production.yml`
   - Useful for overriding default behavior or testing specific configs

2. **Auto-Scan (Default)**
   - If `DBHUB_CONFIG` is not set, DB Hub automatically scans `~/.dbhub/`
   - Loads all valid `.yml` and `.yaml` files
   - Files are sorted alphabetically for consistent ordering

### File Filtering

The scanner automatically **excludes**:

- ❌ Hidden files (starting with `.`)
- ❌ Temporary files (starting with `#` or ending with `~`)
- ❌ Backup files (ending with `.bak`, `.backup`)
- ❌ Non-YAML files (not ending with `.yml` or `.yaml`)
- ❌ Invalid YAML files (files that fail parsing)
- ❌ Directories

The scanner **includes**:

- ✅ Valid `.yml` files (e.g., `config.yml`, `production.yml`)
- ✅ Valid `.yaml` files (e.g., `config.yaml`, `staging.yaml`)
- ✅ Files that can be successfully parsed as DB Hub configuration

### Example Directory Structure

```
~/.dbhub/
├── config.yml              # ✅ Loaded
├── production.yml          # ✅ Loaded
├── staging.yml             # ✅ Loaded
├── test.yml                # ✅ Loaded
├── config.yml~             # ❌ Skipped (temporary file)
├── .hidden.yml             # ❌ Skipped (hidden file)
├── backup.yml.bak          # ❌ Skipped (backup file)
├── invalid.yml             # ❌ Skipped (invalid YAML)
├── notes.txt               # ❌ Skipped (not YAML)
└── scripts/                # ❌ Skipped (directory)
```

## Usage

### GUI Application (macOS)

The GUI application now works out-of-the-box without any environment variable setup:

1. Place your config files in `~/.dbhub/`
2. Launch DBHub from Applications
3. All databases from all config files will appear in the Connect menu

### CLI Tool

The CLI tool benefits from the same auto-scan feature:

```bash
# No need to set DBHUB_CONFIG anymore
dbhub context

# All config files in ~/.dbhub/ are automatically loaded
dbhub connect my-database

# Override with environment variable if needed
export DBHUB_CONFIG=~/.dbhub/config.yml:~/.dbhub/production.yml
dbhub context
```

## Benefits

### For GUI Users
- ✅ No environment variable configuration needed
- ✅ Works seamlessly when launched from Finder or Dock
- ✅ No need to edit shell configuration files

### For CLI Users
- ✅ Simpler setup - just drop files in `~/.dbhub/`
- ✅ Multiple environments automatically organized
- ✅ Backward compatible with existing setups

### For Both
- ✅ Unified behavior between CLI and GUI
- ✅ Consistent configuration loading
- ✅ Easy to manage multiple environments

## Migration Guide

### From DBHUB_CONFIG to Auto-Scan

**Before (using environment variable):**
```bash
# In ~/.zshrc or ~/.bashrc
export DBHUB_CONFIG=~/.dbhub/config.yml:~/.dbhub/production.yml:~/.dbhub/staging.yml
```

**After (auto-scan):**
```bash
# Just ensure all files are in ~/.dbhub/
ls ~/.dbhub/
# config.yml
# production.yml
# staging.yml
# All automatically loaded!
```

### Keeping Environment Variable

If you prefer to use `DBHUB_CONFIG`, it still works:

```bash
# Still supported for backward compatibility
export DBHUB_CONFIG=~/.dbhub/config.yml:~/.dbhub/production.yml
dbhub context  # Only loads specified files
```

## Configuration File Merging

When multiple config files are loaded, they are merged in this order:

1. Files are sorted alphabetically
2. Databases from all files are collected into a single list
3. Templates from all files are merged
4. Aliases are built from all databases (duplicates are warned)

### Example

**config.yml:**
```yaml
databases:
  - alias: local-mysql
    db_type: mysql
    dsn: mysql://root@localhost:3306/test
    env: local
```

**production.yml:**
```yaml
databases:
  - alias: prod-mysql
    db_type: mysql
    dsn: mysql://user@prod.example.com:3306/prod
    env: production
```

**Result:** Both `local-mysql` and `prod-mysql` are available.

## Troubleshooting

### GUI Shows No Databases

**Check:** Do config files exist?
```bash
ls ~/.dbhub/*.yml
```

**Check:** Are config files valid?
```bash
# Try to load config manually
dbhub context
```

**Check:** GUI logs (if running from terminal):
```bash
/Applications/DBHub.app/Contents/MacOS/DBHub
```

### Config File Not Loading

**Check:** File extension
```bash
# Must be .yml or .yaml (not .yaml.txt)
mv config.yaml.txt config.yaml
```

**Check:** File validity
```bash
# Ensure it's valid YAML
cat ~/.dbhub/config.yml
```

**Check:** File isn't hidden/temporary
```bash
# Remove leading dots or tildes
mv .config.yml config.yml
mv config.yml~ config.yml
```

### Override Auto-Scan

If you need to load specific files only:
```bash
export DBHUB_CONFIG=~/.dbhub/config.yml
dbhub context
```

## Implementation Details

- **Location:** `core/src/config.rs`
- **Function:** `get_config_paths()`, `scan_config_directory()`, `is_valid_config_file()`
- **Behavior:**
  1. Check `DBHUB_CONFIG` environment variable
  2. If not set, scan `~/.dbhub/` directory
  3. Filter and validate files
  4. Sort alphabetically
  5. Return sorted list of valid config paths

## Future Enhancements

Potential improvements for future versions:

- [ ] Support for custom config directory via config file
- [ ] Config file priority/ordering system
- [ ] Reload config when files change
- [ ] GUI indicator showing which config file each database comes from
- [ ] Config validation GUI with error messages
