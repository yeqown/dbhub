use crate::config::Config;
use crate::tools;
use clap::{Args, CommandFactory, Parser, Subcommand, ValueHint};
use clap_complete::{generate, Shell};
use color_eyre::eyre::Result;
use std::cmp::min;
use std::io;

/// 替换为实际的包描述
#[derive(Parser)]
#[command(author, version, about)]
#[command(name = "dbhub")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Connect to a database using environment and database name
    #[command(alias = "c")]
    Connect {
        /// Connection alias
        #[arg(value_hint = ValueHint::Other, required = true)]
        alias: String,
    },
    /// Manage database connection contexts
    #[command(alias = "e")]
    Context(ContextArgs),
    /// Generate shell completion scripts
    #[command(alias = "comp")]
    Completion {
        /// Shell type
        #[arg(value_enum)]
        shell: Shell,
    },
    /// Internal command for completion suggestions (hidden)
    #[command(hide = true)]
    CompletionSuggestions {
        /// Type of suggestions to generate
        #[arg()]
        suggestion_type: String,
    },
}


#[derive(Args)]
pub struct ContextArgs {
    /// Generate default config file
    #[arg(long, action)]
    pub generate: bool,
    /// Environment name
    #[arg(long, num_args = 1)]
    pub filter_env: Option<String>,
    /// Database type (mysql, mongodb, redis, redis-sentinel)
    #[arg(long, num_args = 1)]
    pub filter_db_type: Option<String>,
    /// Alias name
    #[arg(long, num_args = 1)]
    pub filter_alias: Option<String>,
    /// Output format control: with_dsn
    #[arg(long)]
    pub with_dsn: bool,
    /// Output format control: with_annotations
    #[arg(long)]
    pub with_annotations: bool,
}

pub fn handle_completion_suggestions(cfg: &Config, suggestion_type: &str) -> Result<()> {
    match suggestion_type {
        "aliases" => {
            let mut aliases = cfg.get_all_aliases();
            aliases.sort();

            for alias in aliases {
                println!("{alias}");
            }
        }
        _ => {
            // Unknown suggestion type, return empty
        }
    }
    Ok(())
}

pub fn handle_completion(shell: Shell) -> Result<()> {
    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "dbhub", &mut io::stdout());
    Ok(())
}

pub fn handle_connect(cfg: &Config, alias: &String) -> Result<()> {
    let db_index = cfg.aliases.get(alias).ok_or_else(|| {
        let similar_alias = find_similar_alias(alias, cfg);
        // warn!("Alias '{}' not found, maybe {}?", alias, similar_alias);
        color_eyre::eyre::eyre!("Alias '{}' not found, maybe {}?", alias, similar_alias)
    })?;

    let db = cfg.get_database_by_index(db_index).unwrap();

    tools::connect(db, cfg)
}

fn find_similar_alias(alias: &str, cfg: &Config) -> String {
    // find the most similar alias of the given alias by calculating the Levenshtein distance
    let mut min_distance = usize::MAX;
    let mut similar_alias = String::new();

    cfg.aliases.keys().for_each(|_alias| {
        let distance = levenshtein_distance(alias, _alias);
        if distance < min_distance {
            min_distance = distance;
            similar_alias = _alias.clone();
        }
    });

    similar_alias
}

fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let mut dp = vec![vec![0; s2.len() + 1]; s1.len() + 1];
    for i in 0..=s1.len() {
        for j in 0..=s2.len() {
            if i == 0 {
                dp[i][j] = j;
            } else if j == 0 {
                dp[i][j] = i;
            } else {
                dp[i][j] = min(
                    dp[i - 1][j - 1] + if s1.chars().nth(i - 1) == s2.chars().nth(j - 1) { 0 } else { 1 },
                    min(dp[i - 1][j] + 1, dp[i][j - 1] + 1),
                );
            }
        }
    }
    dp[s1.len()][s2.len()]
}

#[test]
fn test_levenshtein_distance() {
    assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
    assert_eq!(levenshtein_distance("hello", "world"), 4);
    assert_eq!(levenshtein_distance("abc", "abc"), 0);
    assert_eq!(levenshtein_distance("abc", "abcd"), 1);
    assert_eq!(levenshtein_distance("abc", "ab"), 1);
}