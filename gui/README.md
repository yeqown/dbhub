# DB Hub GUI

macOS menu bar GUI for DB Hub.

## Features

- Menu bar integration for quick access
- Connect to databases with one click
- Manage database contexts
- Edit configuration file
- Opens connections in new Terminal windows

## Development

```bash
# Build
cargo build

# Run
cargo run

# Build release bundle
cargo tauri build
```

## Configuration

The GUI uses the same configuration file as the CLI: `~/.dbhub/config.yml`

## Usage

1. Run the application
2. Click the menu bar icon
3. Select Connect > [Environment] > [Database]
4. A new Terminal window opens with the database CLI
