# DB Hub

A command-line tool for managing multiple database connections across different environments. Supports MySQL, MongoDB, DocumentDB, Doris, and Redis databases.

[中文文档](README.zh.md)

## Features

- Multi-environment configuration management (dev, test, prod, etc.)
- Multiple database support (MySQL, MongoDB, DocumentDB, Doris, Redis)
- Customizable connection string templates
- Automatic configuration format validation
- Connection alias support
- Cross-platform support (Windows, Linux, macOS)
- Automatic database client tool installation

## Installation

Download the appropriate binary for your system from [GitHub Releases](https://github.com/your-username/db-hub/releases):

- Linux: `db-hub-linux-amd64.tar.gz` or `db-hub-linux-arm64.tar.gz`
- macOS: `db-hub-darwin-amd64.tar.gz` or `db-hub-darwin-arm64.tar.gz`
- Windows: `db-hub-windows-amd64.exe.zip`

## Usage

### Adding a Database Connection

```bash
db-hub add -e <environment> -n <database-name> -t <database-type> -u <connection-url> [-a <alias>]

# Example
db-hub add -e dev -n myapp -t mysql -u "mysql://user:pass@localhost:3306/myapp" -a dev-db
```

### Connecting to a Database

Using environment and database name:
```bash
db-hub connect -e <environment> -d <database-name>
```

Or using an alias:
```bash
db-hub connect -a <alias>
```

### Listing All Configurations

```bash
db-hub list
```

### Customizing Connection String Templates

```bash
db-hub template -t <database-type> -t <template>

# Example
db-hub template -t mysql -t "mysql://{user}:{password}@{host}:{port}/{database}?charset=utf8mb4"
```

### Installing Database Client Tools

```bash
db-hub install -t <tool-name>

# Supported tools
db-hub install -t mycli      # MySQL/Doris client
db-hub install -t mongosh    # MongoDB/DocumentDB client
db-hub install -t redis-cli  # Redis client
```

## Configuration File

The configuration file is stored at `~/.db-hub/config.yml` with the following format:

```yaml
environments:
  dev:
    databases:
      myapp:
        db_type: mysql
        url: mysql://user:pass@localhost:3306/myapp
  prod:
    databases:
      analytics:
        db_type: mongodb
        url: mongodb://user:pass@prod:27017/analytics

templates:
  mysql: "mysql://{user}:{password}@{host}:{port}/{database}"
  mongodb: "mongodb://{user}:{password}@{host}:{port}/{database}"
  redis: "redis://{user}:{password}@{host}:{port}/{database}"

aliases:
  dev-db: 
    env: dev
    db: myapp
```

## License

MIT