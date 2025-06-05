use crate::config::{Config, Database, Template};
use color_eyre::eyre;
use shell_words;
use std::process::Command;
use tracing::{info, warn};
use which::which;


pub const MYSQL: &str = "mysql";
const DORIS: &str = "doris";

pub const MONGODB: &str = "mongodb";
const DOCUMENTDB: &str = "documentdb";

pub const REDIS: &str = "redis";
// TODO(@yeqown): add redis-sentinel support.
const REDIS_SENTINEL: &str = "redis-sentinel";

// TODO(@yeqown): add memcached support.
#[warn(unused)]
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
pub fn connect(db: &Database, cfg: &Config) -> Result<(), eyre::Report> {
    let command = match db.db_type.as_str() {
        MYSQL | DORIS => MYSQL_CLI_COMMAND,
        MONGODB | DOCUMENTDB => MONGO_CLI_COMMAND,
        REDIS | REDIS_SENTINEL => REDIS_CLI_COMMAND,
        _ => return Err(eyre::eyre!("Unsupported database type: {}", db.db_type)),
    };

    let template = cfg.templates.get(command);
    if template.is_none() {
        return Err(eyre::eyre!("Template not found for CLI: {}", command));
    }
    let cli_template = template.unwrap();

    info!("Connecting to database '{}' using command '{}'", db.alias, command);

    // Whether the command exists
    which(command).map_err(|_| {
        eyre::eyre!("Command '{}' not found. Use 'dbhub install -t {}' to install it.", command, command)
    })?;


    let args = build_cli_command(db, cli_template)?;
    info!("Running command: \n\n\t💻> {} {}\n", command, args.join(" "));

    // DONE(@yeqown): open another shell to execute the interactive command.
    //  and then exit the current shell.
    Command::new(command).args(args)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()?
        .wait()?;

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
fn build_cli_command(db: &Database, template: &Template) -> Result<Vec<String>, eyre::Report> {
    let variables = db.variables(template.dsn.as_str());
    let s = crate::template::fill_template(template.cli.as_str(), &variables);
    if let Some(s) = s {
        let args = shell_words::split(&s)?;

        // if the CLI is start with CLI command, then remove it.
        if let Some(first_arg) = args.first() {
            if first_arg == MYSQL_CLI_COMMAND || first_arg == MONGO_CLI_COMMAND || first_arg == REDIS_CLI_COMMAND {
                return Ok(args[1..].to_vec());
            }
        }

        return Ok(args);
    }

    Err(
        eyre::eyre!("Failed to parse the connection string: {} as template: {}", db.dsn, template.dsn)
    )
}

pub fn install_tool(tool: &str) -> Result<(), eyre::Report> {
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

fn install_mysql() -> Result<(), eyre::Report> {
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

fn install_mongosh() -> Result<(), eyre::Report> {
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

fn install_redis_cli() -> Result<(), eyre::Report> {
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
