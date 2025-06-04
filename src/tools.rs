use color_eyre::eyre::Result;
use shell_words;
use std::process::Command;
use tracing::info;
use crate::config::{Database};
use which::which;

// TODO(@yeqown): add redis-sentinel support.
// TODO(@yeqown): add memcached support.

pub fn connect(db: &Database) -> Result<()> {
    let command = match db.db_type.as_str() {
        "mysql" | "doris" => "mysql",
        "mongodb" | "documentdb" => "mongosh",
        "redis" => "redis-cli",
        _ => return Err(color_eyre::eyre::eyre!("Unsupported database type: {}", db.db_type)),
    };

    info!("Connecting to database '{}' using command '{}'", db.alias, command);

    // Whether the command exists
    which(command).map_err(|_| {
        color_eyre::eyre::eyre!("Command '{}' not found. Use 'dbhub install -t {}' to install it.", command, command)
    })?;

    let args = shell_words::split(&db.url)?;
    let status = Command::new(command)
        .args(args)
        .status()?;

    if !status.success() {
        return Err(color_eyre::eyre::eyre!("Failed to connect to database"));
    }

    Ok(())
}

pub fn install_tool(tool: &str) -> Result<()> {
    // detect the client tool is already installed.
    // if installed, just return.
    // if not, try to install it.
    if which(tool).is_ok() {
        println!("The '{}' command is already installed.", tool);
        return Ok(());
    }

    match tool {
        "mysql" => install_mysql()?,
        "mongosh" => install_mongosh()?,
        "redis-cli" => install_redis_cli()?,
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
