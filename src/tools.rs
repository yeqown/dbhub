use crate::config::{Config, Database, Template};
use color_eyre::eyre::{eyre, Result};
use shell_words;
use std::process::Command;
use tracing::{info, warn};
use which::which;

pub const MYSQL: &str = "mysql";

pub const MONGODB: &str = "mongo";

pub const REDIS: &str = "redis";

// TODO(@yeqown): add memcached support.
#[warn(unused)]
const MEMCACHED: &str = "memcached";

const MYSQL_CLI_COMMAND: &str = "mysqlsh";
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
    let mut commanded_cli = match db.db_type.as_str() {
        MYSQL => MYSQL_CLI_COMMAND,
        MONGODB => MONGO_CLI_COMMAND,
        REDIS => REDIS_CLI_COMMAND,
        _ => return Err(eyre!("Unsupported database type: {}", db.db_type)),
    };

    let template = cfg.templates.get(db.db_type.as_str());
    if template.is_none() {
        return Err(eyre!("No template found for database type: {}", db.db_type));
    }
    let tpl = template.unwrap();

    let args = build_cli_command(db, tpl)?;
    info!("Running command: \n\n\t💻 -> {}\n", args.join(" "));

    // Extract the command from the first argument
    if let Some(template_cli) = args.first() {
        if template_cli.ne(MYSQL_CLI_COMMAND) {
            commanded_cli = template_cli;
        }
    }

    // Whether the command exists
    which(commanded_cli).map_err(|_| {
        eyre!("Command '{}' not found. please install it or check your PATH.", commanded_cli)
    })?;

    // DONE(@yeqown): open another shell to execute the interactive command.
    //  and then exit the current shell.
    Command::new(commanded_cli).args(&args[1..])
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()?
        .wait()?;

    info!("The connection has been closed.");

    Ok(())
}

fn build_cli_command(db: &Database, template: &Template) -> Result<Vec<String>> {
    let variables = db.variables(template.dsn.as_str())?;

    // TODO(@yeqown): use lua to enhance the flexibility of customize the connection command.

    let s = crate::template::fill_template(template.cli.as_str(), &variables);
    if let Some(s) = s {
        let args = shell_words::split(&s)?;

        return Ok(args);
    }

    Err(
        eyre!("Failed to parse the connection string: {} as template: {}", db.dsn, template.dsn)
    )
}

// pub fn install_tool(tool: &str) -> Result<(), eyre::Report> {
//     if which(tool).is_ok() {
//         println!("The '{}' command is already installed.", tool);
//         return Ok(());
//     }
//
//     match tool {
//         MYSQL_CLI_COMMAND => install_mysql()?,
//         MONGO_CLI_COMMAND => install_mongosh()?,
//         REDIS_CLI_COMMAND => install_redis_cli()?,
//         _ => return Err(color_eyre::eyre::eyre!("Unsupported tool: {}", tool)),
//     }
//     Ok(())
// }
// fn install_mysql() -> Result<(), eyre::Report> {
//     #[cfg(target_os = "macos")]
//     {
//         Command::new("brew")
//             .args(["install", "mysql"])
//             .status()?;
//     }
//     #[cfg(target_os = "linux")]
//     {
//         Command::new("sudo")
//             .args(["apt-get", "install", "-y", "mysql-client"])
//             .status()?;
//     }
//     #[cfg(target_os = "windows")]
//     {
//         Command::new("choco")
//             .args(["install", "mysql", "-y"])
//             .status()?;
//     }
//     Ok(())
// }
//
// fn install_mongosh() -> Result<(), eyre::Report> {
//     #[cfg(target_os = "macos")]
//     {
//         Command::new("brew")
//             .args(["install", "mongosh"])
//             .status()?;
//     }
//     #[cfg(target_os = "linux")]
//     {
//         // 为 Linux 添加 MongoDB 仓库并安装 mongosh
//         Command::new("curl")
//             .args(["-fsSL", "https://www.mongodb.org/static/pgp/server-6.0.asc", "-o", "/tmp/mongodb.asc"])
//             .status()?;
//         Command::new("sudo")
//             .args(["apt-key", "add", "/tmp/mongodb.asc"])
//             .status()?;
//         Command::new("echo")
//             .arg("deb [ arch=amd64,arm64 ] https://repo.mongodb.org/apt/ubuntu focal/mongodb-org/6.0 multiverse")
//             .stdout(std::process::Stdio::piped())
//             .spawn()?
//             .stdout
//             .ok_or_else(|| color_eyre::eyre::eyre!("Failed to get stdout"))?;
//         Command::new("sudo")
//             .args(["apt-get", "update"])
//             .status()?;
//         Command::new("sudo")
//             .args(["apt-get", "install", "-y", "mongodb-mongosh"])
//             .status()?;
//     }
//     #[cfg(target_os = "windows")]
//     {
//         Command::new("choco")
//             .args(["install", "mongodb-shell", "-y"])
//             .status()?;
//     }
//     Ok(())
// }
//
// fn install_redis_cli() -> Result<(), eyre::Report> {
//     #[cfg(target_os = "macos")]
//     {
//         Command::new("brew")
//             .args(["install", "redis"])
//             .status()?;
//     }
//     #[cfg(target_os = "linux")]
//     {
//         Command::new("sudo")
//             .args(["apt-get", "install", "-y", "redis-tools"])
//             .status()?;
//     }
//     #[cfg(target_os = "windows")]
//     {
//         Command::new("choco")
//             .args(["install", "redis-cli", "-y"])
//             .status()?;
//     }
//     Ok(())
// }
