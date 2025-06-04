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

Download the appropriate binary for your system from [GitHub Releases](https://github.com/your-username/dbhub/releases):

- Linux: `dbhub-linux-amd64.tar.gz` or `dbhub-linux-arm64.tar.gz`
- macOS: `dbhub-darwin-amd64.tar.gz` or `dbhub-darwin-arm64.tar.gz`
- Windows: `dbhub-windows-amd64.exe.zip`

## Usage

### Adding a Database Connection

```bash
dbhub add -e <environment> -n <database-name> -t <database-type> -u <connection-url> [-a <alias>]

# Example
dbhub add -e dev -n myapp -t mysql -u "mysql://user:pass@localhost:3306/myapp" -a dev-db
```

### Connecting to a Database

Using environment and database name:
```bash
dbhub connect -e <environment> -d <database-name>
```

Or using an alias:
```bash
dbhub connect -a <alias>
```

### Listing All Configurations

```bash
dbhub list
```

### Customizing Connection String Templates

```bash
dbhub template -t <database-type> -t <template>

# Example
dbhub template -t mysql -t "mysql://{user}:{password}@{host}:{port}/{database}?charset=utf8mb4"
```

### Installing Database Client Tools

```bash
dbhub install -t <tool-name>

# Supported tools
dbhub install -t mycli      # MySQL/Doris client
dbhub install -t mongosh    # MongoDB/DocumentDB client
dbhub install -t redis-cli  # Redis client
```

## Configuration File

The configuration file is stored at `~/.dbhub/config.yml` with the following format:

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