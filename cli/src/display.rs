use console::{style, StyledObject};
use dbhub_core::{Config, Database};
use std::collections::BTreeMap;

/// Output format options for listing connections.
#[derive(Debug, Default)]
pub struct ListFormat {
    pub with_desc: bool,
    pub with_dsn: bool,
    pub with_annotations: bool,
}

impl ListFormat {
    pub fn from_args(args: &super::cli::ContextArgs) -> Self {
        ListFormat {
            with_dsn: args.with_dsn,
            with_annotations: args.with_annotations,
            ..Default::default()
        }
    }
}

/// Filter options for listing connections.
#[derive(Debug, Default)]
pub struct Filter {
    pub env: Option<String>,
    pub db_type: Option<String>,
    pub alias: Option<String>,
}

impl Filter {
    pub fn from_args(args: &super::cli::ContextArgs) -> Self {
        Filter {
            env: args.filter_env.clone(),
            db_type: args.filter_db_type.clone(),
            alias: args.filter_alias.clone(),
        }
    }
}

/// Options for listing connections.
#[derive(Debug, Default)]
pub struct ListOptions {
    pub filter: Filter,
    pub format: ListFormat,
}

impl ListOptions {
    pub fn from_args(args: &super::cli::ContextArgs) -> Self {
        ListOptions {
            filter: Filter::from_args(args),
            format: ListFormat::from_args(args),
        }
    }
}

/// List all database connections with optional filtering.
pub fn list_connections(config: &Config, opts: &ListOptions) {
    println!("{}", style("Databases:").bold());

    let mut found_databases = 0;

    // Group databases by env and db_type
    let mut grouped_databases: BTreeMap<&str, BTreeMap<&str, Vec<&Database>>> = BTreeMap::new();

    for (i, db) in config.databases.iter().enumerate() {
        // Apply environment filter
        if let Some(ref specified_env) = opts.filter.env {
            if db.env != *specified_env {
                continue;
            }
        }

        // Apply alias filter
        if let Some(ref specified_alias) = opts.filter.alias {
            if db.alias != *specified_alias {
                continue;
            }
        }

        // Apply db_type filter
        if let Some(ref specified_db_type) = opts.filter.db_type {
            if db.db_type != *specified_db_type {
                continue;
            }
        }

        found_databases += 1;

        grouped_databases
            .entry(&db.env)
            .or_default()
            .entry(&db.db_type)
            .or_default()
            .push(config.databases.get(i).unwrap());
    }

    print_databases(grouped_databases, opts);

    if found_databases == 0 {
        println!("{}", style("No databases found.").red());
    }
}

fn print_databases(
    grouped_databases: BTreeMap<&str, BTreeMap<&str, Vec<&Database>>>,
    opts: &ListOptions,
) {
    for (env, db_type_map) in grouped_databases {
        let styled_env: StyledObject<&str> = style(env).blue().bold();
        println!("  {styled_env}");

        for (db_type, db_list) in db_type_map {
            let styled_db_type: StyledObject<&str> = style(db_type).green().bold();
            println!("    {styled_db_type}");

            let mut is_first = true;

            for db in db_list {
                if !is_first {
                    println!();
                }

                let alias = format!("🚀 Alias: {}", style(&db.alias).bold());
                println!("\t{alias}");

                if opts.format.with_desc {
                    let desc = format!(
                        "📜 Desc : {}",
                        style(db.description.clone().unwrap_or_else(|| "No description".to_string()))
                            .dim()
                    );
                    println!("\t{desc}");
                }

                if opts.format.with_dsn {
                    let dsn = format!("🔗 DSN : {}", style(&db.dsn).dim());
                    println!("\t{dsn}");
                }

                if opts.format.with_annotations {
                    if let Some(annos) = &db.annotations {
                        println!("\t{}", style("📝 Annotations:").bold());
                        for (key, value) in annos {
                            let anno = format!("-> \"{key}\": {value}");
                            println!("\t\t{anno}");
                        }
                    }
                }

                is_first = false;
            }
        }
    }
}
