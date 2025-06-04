use crate::config::{Config, Database, Template};
use color_eyre::eyre::Result;
use shell_words;
use std::process::Command;
use tracing::info;
use which::which;


pub const MYSQL: &str = "mysql";
const DORIS: &str = "doris";

pub const MONGODB: &str = "mongodb";
const DOCUMENTDB: &str = "documentdb";

pub const REDIS: &str = "redis";
// TODO(@yeqown): add redis-sentinel support.
const REDIS_SENTINEL: &str = "redis-sentinel";

// TODO(@yeqown): add memcached support.
const MEMCACHED: &str = "memcached";


const MYSQL_CLI_COMMAND: &str = "mysql";
const MONGO_CLI_COMMAND: &str = "mongosh";
const REDIS_CLI_COMMAND: &str = "redis-cli";

/// Connect to a database using environment and database name
///
/// # Arguments
///
/// * `db` - The database connection information
///
/// # Returns
///
/// * `Result<()>` - Returns `Ok(())` if the connection is successful, otherwise returns an error
pub fn connect(db: &Database, cfg: &Config) -> Result<()> {
    let command = match db.db_type.as_str() {
        MYSQL | DORIS => MYSQL_CLI_COMMAND,
        MONGODB | DOCUMENTDB => MONGO_CLI_COMMAND,
        REDIS | REDIS_SENTINEL => REDIS_CLI_COMMAND,
        _ => return Err(color_eyre::eyre::eyre!("Unsupported database type: {}", db.db_type)),
    };

    info!("Connecting to database '{}' using command '{}'", db.alias, command);

    // Whether the command exists
    which(command).map_err(|_| {
        color_eyre::eyre::eyre!("Command '{}' not found. Use 'dbhub install -t {}' to install it.", command, command)
    })?;

    let mut cmd = Command::new(command);
    let args = build_cli_command(db, &cfg.templates.get(&db.db_type).unwrap());
    let cli = cmd.args(args);

    // print the command and args
    info!("Running command: {:?}", cli);

    // TODO(@yeqown): open another shell to execute the interactive command.
    //  and then exit the current shell.

    Ok(())
}

/// Parse the database connection string and build command line parameters
/// Example:
///
/// Mysql:
///    dsn: mysql://root:root@localhost:3306/test?parseTime=true
///    db_type: mysql
/// Mysql Template:
///    dsn: mysql://{user}:{password}@{host}:{port}/{database}?{options}
///    cli: mysql -u {user} -p{password} -h {host} -P {port} {database}
///
/// Output:
///    mysql -u root -proot -h localhost -P 3306 test
///
fn build_cli_command(db: &Database, template: &Template) -> Vec<String> {
    let variables = db.variables(template);
    vec![variables.get("dsn").take().unwrap().to_string()]
}

pub fn install_tool(tool: &str) -> Result<()> {
    if which(tool).is_ok() {
        println!("The '{}' command is already installed.", tool);
        return Ok(());
    }

    match tool {
        MYSQL_CLI_COMMAND => install_mysql()?,
        MONGO_CLI_COMMAND => install_mongosh()?,
        REDIS_CLI_COMMAND => install_redis_cli()?,
        _ => return Err(color_eyre::eyre::eyre!("Unsupported tool: {}", tool)),
    }
    Ok(())
}

// 添加安装 mysql 的函数
fn install_mysql() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        Command::new("brew")
            .args(["install", "mysql"])
            .status()?;
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("sudo")
            .args(["apt-get", "install", "-y", "mysql-client"])
            .status()?;
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("choco")
            .args(["install", "mysql", "-y"])
            .status()?;
    }
    Ok(())
}

fn install_mongosh() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        Command::new("brew")
            .args(["install", "mongosh"])
            .status()?;
    }
    #[cfg(target_os = "linux")]
    {
        // 为 Linux 添加 MongoDB 仓库并安装 mongosh
        Command::new("curl")
            .args(["-fsSL", "https://www.mongodb.org/static/pgp/server-6.0.asc", "-o", "/tmp/mongodb.asc"])
            .status()?;
        Command::new("sudo")
            .args(["apt-key", "add", "/tmp/mongodb.asc"])
            .status()?;
        Command::new("echo")
            .arg("deb [ arch=amd64,arm64 ] https://repo.mongodb.org/apt/ubuntu focal/mongodb-org/6.0 multiverse")
            .stdout(std::process::Stdio::piped())
            .spawn()?
            .stdout
            .ok_or_else(|| color_eyre::eyre::eyre!("Failed to get stdout"))?;
        Command::new("sudo")
            .args(["apt-get", "update"])
            .status()?;
        Command::new("sudo")
            .args(["apt-get", "install", "-y", "mongodb-mongosh"])
            .status()?;
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("choco")
            .args(["install", "mongodb-shell", "-y"])
            .status()?;
    }
    Ok(())
}

fn install_redis_cli() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        Command::new("brew")
            .args(["install", "redis"])
            .status()?;
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("sudo")
            .args(["apt-get", "install", "-y", "redis-tools"])
            .status()?;
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("choco")
            .args(["install", "redis-cli", "-y"])
            .status()?;
    }
    Ok(())
}
