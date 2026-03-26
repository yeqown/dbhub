# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

DB Hub is a Rust CLI tool for managing multi-environment database connections. It supports MySQL, MongoDB, Redis, Redis-Sentinel, and Memcached databases through a Lua-based extensibility model.

**Core architecture**: The tool parses DSN templates to extract connection variables, then uses Lua scripts to generate database-specific CLI commands (e.g., `mysqlsh`, `mongosh`, `redis-cli`).

## Build Commands

```bash
# Development build
cargo build

# Release build (for current platform)
cargo build --release

# Install to /usr/local/bin (macOS only)
make install

# Run tests (inline with modules)
cargo test

# Publish to crates.io
cargo publish --dry-run  # test first
cargo publish
```

## Code Architecture

### Module Structure

- **main.rs**: Entry point, CLI parsing, command routing
- **cli.rs**: CLI command definitions, completion handling, fuzzy alias matching
- **config.rs**: Configuration loading/validation, alias indexing, connection listing
- **tools.rs**: Database connection logic, Lua script execution
- **template.rs**: DSN template parsing and variable extraction (token-based)
- **embedded.rs**: Embedded resources (configs/, scripts/) using rust-embed

### Key Data Flow

1. User runs `dbhub connect <alias>` → CLI parses alias
2. Config loads from `~/.dbhub/config.yml` (or `DBHUB_CONFIG` env var)
3. Alias resolves to `Database` config
4. `Database.variables()` parses DSN against template to extract host, port, user, etc.
5. Lua script (`~/.dbhub/{db_type}.lua`) generates CLI command with variables
6. Command executed via `std::process::Command`

### Configuration System

- **Default path**: `~/.dbhub/config.yml`
- **Multi-config**: Set `DBHUB_CONFIG=/path/to/config1.yml:/path/to/config2.yml`
- **Merging**: Multiple configs merge (databases + templates)
- **Alias resolution**: In-memory HashMap for O(1) lookup
- **Environments**: Group databases by `env` field for filtering

### Template Variable System

The `template.rs` module implements a token-based parser:

- **Template format**: `mysql://{user}:{password}@{host}:{port}/{database}`
- **Parsing**: `analyze()` → `Vec<TemplateToken>` (Literal or Variable)
- **Variable extraction**: `parse_variables()` matches input against template
- **Tests**: Comprehensive inline tests in template.rs

**Note**: Variables are extracted by matching literals between `{variable}` placeholders. Consecutive variables without literal separators (`{var1}{var2}`) are not supported.

### Lua Integration

Each database type has a corresponding Lua script in `scripts/`:

- **Input**: `dbhub` table with `variables`, `annotations`, `runtime_args`, `last_output_lines`, `count`
- **Output**: Table with `command_with_args` (string) and `again` (bool)
- **Looping**: If `again=true`, command output is captured and passed to next iteration (max 5 iterations)
- **Runtime args**: Users can pass `-- --db=test` to override Lua script behavior

**Embedded scripts**: Scripts are embedded at compile time. If user's `~/.dbhub/{db_type}.lua` is missing, the embedded script is copied there.

### Shell Completion

- Uses `clap_complete` for zsh, bash, fish, PowerShell
- Custom completion suggestions via hidden `CompletionSuggestions` command
- Alias completion for the `connect` subcommand

## Development Guidelines

### Adding New Database Support

1. Create `scripts/{db_type}.lua` in the project
2. Add template entry to sample config (`configs/sample.yml`):
   ```yaml
   templates:
     {db_type}:
       dsn: {template_format}
   ```
3. Script will be auto-copied to `~/.dbhub/{db_type}.lua` on first use
4. No Rust code changes needed for basic support

### Error Handling

- Uses `color-eyre` for pretty error messages
- Errors return early (Guard Clauses pattern)
- Missing CLI tools detected via `which::which()`

### Testing

- Tests are inline with modules using `#[cfg(test)]`
- Run with `cargo test`
- Focus areas: template parsing, variable extraction, CLI routing

### Logging

- Uses `tracing` crate
- Default level: WARN
- Set via `RUST_LOG=debug` environment variable

## Release Process

1. Bump version in `Cargo.toml`
2. Push git tag: `git tag v{version}` and `git push --tags`
3. GitHub Actions builds for macOS (arm64/amd64) and Linux (amd64)
4. Artifacts attached to GitHub Release automatically
5. Manual `cargo publish` for crates.io

## File Locations Reference

- **User config**: `~/.dbhub/config.yml`
- **Lua scripts**: `~/.dbhub/{db_type}.lua`
- **Embedded configs**: `configs/sample.yml` (build time)
- **Embedded scripts**: `scripts/*.lua` (build time)
