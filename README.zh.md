# DB Hub

一个用于管理多环境、多数据库连接的命令行工具。支持 MySQL、MongoDB、DocumentDB、Doris 和 Redis 数据库的连接管理。

## 特性

- 多环境配置管理（开发、测试、生产等）
- 多数据库支持（MySQL、MongoDB、Redis、Redis-Sentinel）
- 连接字符串模板自定义
- 配置格式自动校验
- 支持为连接配置别名
- 跨平台支持（Windows、Linux、macOS）
- Lua 脚本支持自定义命令

## 安装

从 [GitHub Releases](https://github.com/your-username/dbhub/releases) 下载适合你系统的二进制文件：

- Linux: `dbhub-linux-amd64.tar.gz` 或 `dbhub-linux-arm64.tar.gz`
- macOS: `dbhub-darwin-amd64.tar.gz` 或 `dbhub-darwin-arm64.tar.gz`
- Windows: `dbhub-windows-amd64.exe.zip`

## 使用方法

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

## 配置文件

配置文件存储在 `~/.dbhub/config.yml`，格式如下：

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
#   - `metadata`: A Key-Value map of metadata for the database.
databases:
  - alias: my-local-mysql
    db_type: mysql
    dsn: "mysql://root:root@tcp(localhost:3306)/db?parseTime=True"
    env: local
    description: "The local mysql database for quickly testing dbhub CLI."
    metadata:
      mysql: "1"
      version: "8.0.32"
  - alias: my-local-mongo
    db_type: mongo
    dsn: "mongodb://user:password@localhost:27017/db"
    env: local
    description: "The local mongo database for quickly testing dbhub CLI."
    metadata:
      mongo: "1"
      version: "6.0.1"
  - alias: my-local-redis
    db_type: redis
    dsn: "redis://user:password@localhost:6379/0"
    env: local
    description: "The local redis database for quickly testing dbhub CLI."
    metadata:
      redis: "1"
      version: "7.2.1"

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
```

## 许可证

MIT