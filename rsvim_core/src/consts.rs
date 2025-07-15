//! Global constants.

#![allow(non_snake_case)]

use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Duration;

use path_config::PathConfig;

pub mod path_config;

#[cfg(test)]
mod path_config_tests;

/// Mutex locking timeout in seconds, by default is [`u64::MAX`].
///
/// NOTE: This constant can be configured through `RSVIM_MUTEX_TIMEOUT_SECS` environment variable.
pub fn MUTEX_TIMEOUT_SECS() -> u64 {
  static VALUE: OnceLock<u64> = OnceLock::new();

  *VALUE.get_or_init(|| {
    std::env::var("RSVIM_MUTEX_TIMEOUT_SECS")
      .map(|v| v.parse::<u64>().unwrap_or(u64::MAX))
      .unwrap_or(u64::MAX)
  })
}

/// Mutex locking timeout duration, by default is [`u64::MAX`] seconds.
pub fn MUTEX_TIMEOUT() -> Duration {
  Duration::from_secs(MUTEX_TIMEOUT_SECS())
}

/// Buffer size for channels communication, by default is 1000.
///
/// NOTE: This constant can be configured through `RSVIM_CHANNEL_BUF_SIZE` environment variable.
pub fn CHANNEL_BUF_SIZE() -> usize {
  static VALUE: OnceLock<usize> = OnceLock::new();

  *VALUE.get_or_init(|| {
    std::env::var("RSVIM_CHANNEL_BUF_SIZE")
      .map(|v| v.parse::<usize>().unwrap_or(1000_usize))
      .unwrap_or(1000_usize)
  })
}

static PATH_CONFIG_VALUE: OnceLock<PathConfig> = OnceLock::new();

/// User config entry path, it can be either one of following files:
///
/// 1. `$XDG_CONFIG_HOME/rsvim/rsvim.{ts,js}` or `$HOME/.config/rsvim/rsvim.{ts.js}`.
/// 2. `$HOME/.rsvim/rsvim.{ts.js}`
/// 3. `$HOME/.rsvim.{ts.js}`
///
/// NOTE:
/// 1. Typescript file is preferred over javascript, if both exist.
/// 2. The detect priority is from higher to lower: 1st > 2nd > 3rd.
/// 3. The 1st config home is `$XDG_CONFIG_HOME/rsvim`, the 2nd and 3rd config home is
///    `$HOME/.rsvim`.
pub fn CONFIG_ENTRY_PATH() -> Option<PathBuf> {
  PATH_CONFIG_VALUE
    .get_or_init(PathConfig::new)
    .config_entry()
    .clone()
}

/// User config home directory, it can be either one of following directories:
///
/// 1. `$XDG_CONFIG_HOME/rsvim/` or `$HOME/.config/rsvim/`.
/// 2. `$HOME/.rsvim/`
pub fn CONFIG_HOME_PATH() -> Option<PathBuf> {
  PATH_CONFIG_VALUE
    .get_or_init(PathConfig::new)
    .config_home()
    .clone()
}

/// Cache home directory, i.e. `$XDG_CACHE_HOME/rsvim`.
pub fn CACHE_HOME_PATH() -> PathBuf {
  PATH_CONFIG_VALUE
    .get_or_init(PathConfig::new)
    .cache_home()
    .clone()
}

/// Data home directory, i.e. `$XDG_DATA_HOME/rsvim`.
pub fn DATA_HOME_PATH() -> PathBuf {
  PATH_CONFIG_VALUE
    .get_or_init(PathConfig::new)
    .data_home()
    .clone()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mutex_timeout1() {
    assert!(MUTEX_TIMEOUT_SECS() > 0);
  }
}
