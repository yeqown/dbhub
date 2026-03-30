pub mod config;
pub mod connection;
pub mod embedded;
pub mod template;

// Re-export commonly used types for external consumers (CLI, GUI)
pub use config::{Config, Database, Template, InitStatus, InitResult, get_config_dir};
pub use config::{get_config_paths, check_init_status, generate_default_config, loads};

// Re-export connection functions
pub use connection::{connect, build_connect_command, ConnectCommand};

// Re-export template parsing functions (used by Database::variables)
pub use template::{parse_variables, analyze, fill_template};
