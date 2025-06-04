# DB Hub

一个用于管理多环境、多数据库连接的命令行工具。支持 MySQL、MongoDB、DocumentDB、Doris 和 Redis 数据库的连接管理。

## 特性

- 多环境配置管理（开发、测试、生产等）
- 多数据库支持（MySQL、MongoDB、DocumentDB、Doris、Redis）
- 连接字符串模板自定义
- 配置格式自动校验
- 支持为连接配置别名
- 跨平台支持（Windows、Linux、macOS）
- 自动安装数据库客户端工具

## 安装

从 [GitHub Releases](https://github.com/your-username/dbhub/releases) 下载适合你系统的二进制文件：

- Linux: `dbhub-linux-amd64.tar.gz` 或 `dbhub-linux-arm64.tar.gz`
- macOS: `dbhub-darwin-amd64.tar.gz` 或 `dbhub-darwin-arm64.tar.gz`
- Windows: `dbhub-windows-amd64.exe.zip`

## 使用方法

### 添加数据库连接

```bash
dbhub add -e <环境名> -n <数据库名> -t <数据库类型> -u <连接URL> [-a <别名>]

# 示例
dbhub add -e dev -n myapp -t mysql -u "mysql://user:pass@localhost:3306/myapp" -a dev-db
```

### 连接数据库

使用环境和数据库名：
```bash
dbhub connect -e <环境名> -d <数据库名>
```

或使用别名：
```bash
dbhub connect -a <别名>
```

### 列出所有配置

```bash
dbhub list
```

### 自定义连接串模板

```bash
dbhub template -t <数据库类型> -t <模板>

# 示例
dbhub template -t mysql -t "mysql://{user}:{password}@{host}:{port}/{database}?charset=utf8mb4"
```

### 安装数据库客户端工具

```bash
dbhub install -t <工具名>

# 支持的工具
dbhub install -t mycli      # MySQL/Doris 客户端
dbhub install -t mongosh    # MongoDB/DocumentDB 客户端
dbhub install -t redis-cli  # Redis 客户端
```

## 配置文件

配置文件存储在 `~/.dbhub/config.yml`，格式如下：

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

## 许可证

MIT