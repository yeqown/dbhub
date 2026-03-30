//! Connection module - handles database connection command generation and execution.

mod command;
mod executor;
mod lua;

pub use command::{build_connect_command, ConnectCommand};
pub use executor::connect;
