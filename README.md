# DB Hub

[![GitHub release](https://img.shields.io/github/release/yeqown/dbhub.svg?style=flat-square)](https://github.com/yeqown/dbhub/releases/latest)
[![GitHub license](https://img.shields.io/github/license/yeqown/dbhub.svg?style=flat-square)](https://github.com/yeqown/dbhub/blob/main/LICENSE)
[![GitHub stars](https://img.shields.io/github/stars/yeqown/dbhub.svg?style=flat-square)](https://github.com/yeqown/dbhub/stargazers)
[![GitHub forks](https://img.shields.io/github/forks/yeqown/dbhub.svg?style=flat-square)](https://github.com/yeqown/dbhub/network)
[![GitHub issues](https://img.shields.io/github/issues/yeqown/dbhub.svg?style=flat-square)](https://github.com/yeqown/dbhub/issues)
[![GitHub pull requests](https://img.shields.io/github/issues-pr/yeqown/dbhub.svg?style=flat-square)](https://github.com/yeqown/dbhub/pulls)
[![Build Status](https://img.shields.io/github/actions/workflow/status/yeqown/dbhub/release.yml?style=flat-square)](https://github.com/yeqown/dbhub/actions/workflows/release.yml)
[![Coverage Status](https://img.shields.io/codecov/c/github/yeqown/dbhub?style=flat-square)](https://app.codecov.io/gh/yeqown/dbhub)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Contributors](https://img.shields.io/github/contributors/yeqown/dbhub.svg?style=flat-square)](https://github.com/yeqown/dbhub/graphs/contributors)
[![Last Commit](https://img.shields.io/github/last-commit/yeqown/dbhub.svg?style=flat-square)](https://github.com/yeqown/dbhub/commits/main)
[![Dependabot](https://badgen.net/github/dependabot/yeqown/dbhub?icon=dependabot&style=flat-square)](https://github.com/yeqown/dbhub/network/updates)

![demo](./carbon.svg)

A command-line tool for managing multiple database connections across different environments. Supports MySQL, MongoDB,
DocumentDB, Doris, and Redis databases.

[中文文档](README.zh.md)

## Features

- Multi-environment configuration management (dev, test, prod, etc.)
- Multiple database support (MySQL, MongoDB, Redis, Redis-Sentinel)
- Customizable connection string templates
- Automatic configuration format validation
- Connection alias support
- Cross-platform support (Windows, Linux, macOS)
- Lua script support for custom commands

## Installation

```shell
cargo binstall dbhub
# or 
cargo install dbhub 
```

## Shell Completion

dbhub supports shell completion for zsh, bash, fish, and PowerShell.

### Quick Installation

Use the provided installation script:

```shell
# Install zsh completion (default)
./scripts/install-completion.sh

# Install for other shells
./scripts/install-completion.sh bash
./scripts/install-completion.sh fish
./scripts/install-completion.sh powershell
```

### Manual Installation

#### Zsh

```shell
# Generate completion script
dbhub completion zsh > ~/.zsh/completions/_dbhub

# Add to your .zshrc if not already present
echo 'fpath=(~/.zsh/completions $fpath)' >> ~/.zshrc
echo 'autoload -U compinit && compinit' >> ~/.zshrc

# Reload your shell
source ~/.zshrc
```

#### Bash

```shell
# Generate completion script
dbhub completion bash > ~/.bash_completion.d/dbhub

# Add to your .bashrc if not already present
echo 'for f in ~/.bash_completion.d/*; do source $f; done' >> ~/.bashrc

# Reload your shell
source ~/.bashrc
```

#### Fish

```shell
# Generate completion script
dbhub completion fish > ~/.config/fish/completions/dbhub.fish
```

#### PowerShell

```shell
# Generate completion script and add to your profile
dbhub completion powershell >> $PROFILE
```

## Usage

```shell
Usage: dbhub [OPTIONS] [COMMAND]

Commands:
  connect  Connect to a database using environment and database name
  context  Manage database connection contexts
  help     Print this message or the help of the given subcommand(s)

Options:
  -c, --config <CONFIG>  Config file path
  -h, --help             Print help
  -V, --version          Print version
```

## Configuration File

The configuration file is stored at `~/.dbhub/config.yml` with the following format:

```yaml
# This is a sample configuration file for the dbhub CLI.
# You can use this file to configure the CLI to connect to your databases.
# The CLI will look for this file in the following locations:
#   - $HOME/.dbhub/config.yml
# or you can specify the path to the file using the --config flag.
# For more information, see the README.md file.

# `databases` section is a list of databases that you want to connect to.
# Each database has the following fields:
#   - `alias`: The alias of the database.
#     You can use this alias to connect to the database.
#
#   - `db_type`: indicates the type of the database helps dbhub to choose database CLI.
#     Now, dbhub supports `mysql`, `mongo`, `redis`.
#
#   - `dsn`: Connection string of the database which obeys the templates.dsn.
#     For example, the dsn for mysql is mysql://{user}:{password}@tcp({host}:{port})/{database}?{query}
#
#   - `env`: The environment of the database.
#
#   - `description`: A string to describe the database detailed.
#
#   - `annotations`: A Key-Value map of annotations for the database.
databases:
  - alias: my-local-mysql
    db_type: mysql
    dsn: "mysql://root:root@tcp(localhost:3306)/db?parseTime=True"
    env: local
    description: "The local mysql database for quickly testing dbhub CLI."
    annotations:
      mysql: "1"
      version: "8.0.32"
  - alias: my-local-mongo
    db_type: mongo
    dsn: "mongodb://user:password@localhost:27017/db"
    env: local
    description: "The local mongo database for quickly testing dbhub CLI."
    annotations:
      mongo: "1"
      version: "6.0.1"
  - alias: my-local-redis
    db_type: redis
    dsn: "redis://user:password@localhost:6379/0"
    env: local
    description: "The local redis database for quickly testing dbhub CLI."
    annotations:
      redis: "1"
      version: "7.2.1"
  - alias: my-local-redis-sentinel
    db_type: redis-sentinel
    dsn: "redis://user:password@localhost:6379/0"
    env: local
    description: "The local redis sentinel database for quickly testing dbhub CLI."
    annotations:
      redis-sentinel/mastername: "mymaster"
  - alias: my-local-memcached
    db_type: memcached
    dsn: "memcached://localhost:11211,localhost:11212"
    env: local
    description: "The local memcached database for quickly testing dbhub CLI."
    annotations:
      memcached/hash-distribution: "murmur3"

# `templates` section is a list of template related to a specified database type including `dsn` and `cli`.
# Each template has the following fields:
#   - `dsn`: Connection string of the database which obeys the templates.dsn.
#     For example, the dsn for mysql is mysql://{user}:{password}@tcp({host}:{port})/{database}?{query}
#
#   - `cli`: The command to connect to the database.
#     For example, the cli for mysql is mysql -h{host} -P{port} -u{user} -p{password} {database}
templates:
  mysql:
    dsn: mysql://{user}:{password}@tcp({host}:{port})/{database}?{query}
  mongo:
    dsn: mongodb://{user}:{password}@{host}:{port}/{database}?{query}
  redis:
    dsn: redis://{user}:{password}@{host}:{port}/{database}
  memcached:
    dsn: memcached://{servers}
```

## License

MIT