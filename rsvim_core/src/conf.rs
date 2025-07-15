//! Configurations.

use std::path::PathBuf;

// Re-export
pub use path_config::PathConfig;

pub mod path_config;

#[cfg(test)]
mod path_config_tests;
