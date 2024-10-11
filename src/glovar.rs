//! Global constants and (environment) variables.

#![allow(non_snake_case)]

use std::env;
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::glovar::path_config::PathConfig;

pub mod path_config;

/// Mutex locking timeout, by default is [`u64::MAX`].
///
/// NOTE: This constant can be configured through `RSVIM_MUTEX_TIMEOUT` environment variable.
pub fn MUTEX_TIMEOUT() -> u64 {
  static VALUE: OnceLock<u64> = OnceLock::new();

  *VALUE.get_or_init(|| match env::var("RSVIM_MUTEX_TIMEOUT") {
    Ok(v1) => match v1.parse::<u64>() {
      Ok(v2) => v2,
      _ => u64::MAX,
    },
    _ => u64::MAX,
  })
}

/// Buffer size for IO operations such as file, sockets, etc. By default is 8192.
///
/// NOTE: This constant can be configured through `RSVIM_IO_BUF_SIZE` environment variable.
pub fn IO_BUF_SIZE() -> usize {
  static VALUE: OnceLock<usize> = OnceLock::new();

  *VALUE.get_or_init(|| match env::var("RSVIM_IO_BUF_SIZE") {
    Ok(v1) => match v1.parse::<usize>() {
      Ok(v2) => v2,
      _ => 8192_usize,
    },
    _ => 8192_usize,
  })
}

/// Buffer size for channels communication, by default is 1000.
///
/// NOTE: This constant can be configured through `RSVIM_CHANNEL_BUF_SIZE` environment variable.
pub fn CHANNEL_BUF_SIZE() -> usize {
  static VALUE: OnceLock<usize> = OnceLock::new();

  *VALUE.get_or_init(|| match env::var("RSVIM_CHANNEL_BUF_SIZE") {
    Ok(v1) => match v1.parse::<usize>() {
      Ok(v2) => v2,
      _ => 1000_usize,
    },
    _ => 1000_usize,
  })
}

static PATH_CONFIG_VALUE: OnceLock<PathConfig> = OnceLock::new();

/// User config file path, it is detected with following orders:
///
/// 1. `$XDG_CONFIG_HOME/rsvim/rsvim.{ts,js}` or `$HOME/.config/rsvim/rsvim.{ts.js}`.
/// 2. `$HOME/.rsvim/rsvim.{ts.js}`
/// 3. `$HOME/.rsvim.{ts.js}`
///
/// NOTE:
/// 1. Typescript file is preferred over javascript, if both exist.
/// 2. For macOS, the `$XDG_CONFIG_HOME` also detects the `$HOME/.config` folder.
pub fn CONFIG_FILE_PATH() -> Option<PathBuf> {
  PATH_CONFIG_VALUE
    .get_or_init(PathConfig::new)
    .config_file()
    .clone()
}

/// User config directory paths, it contains following directories:
///
/// 1. `$XDG_CONFIG_HOME/rsvim/` or `$HOME/.config/rsvim/`.
/// 2. `$HOME/.rsvim/`
pub fn CONFIG_DIRS_PATH() -> Vec<PathBuf> {
  PATH_CONFIG_VALUE
    .get_or_init(PathConfig::new)
    .config_dirs()
    .clone()
}

/// Cache directory path, i.e. `$XDG_CACHE_HOME/rsvim` or `$HOME/.cache/rsvim`.
pub fn CACHE_DIR_PATH() -> PathBuf {
  PATH_CONFIG_VALUE
    .get_or_init(PathConfig::new)
    .cache_dir()
    .clone()
}

/// Data directory path, i.e. `$XDG_DATA_HOME/rsvim` or `$HOME/.local/share/rsvim`.
pub fn DATA_DIR_PATH() -> PathBuf {
  PATH_CONFIG_VALUE
    .get_or_init(PathConfig::new)
    .data_dir()
    .clone()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mutex_timeout1() {
    unsafe {
      env::set_var("RSVIM_MUTEX_TIMEOUT", "128");
      assert_eq!(MUTEX_TIMEOUT(), 128_u64);
    }
  }

  #[test]
  fn io_buf_size1() {
    assert!(IO_BUF_SIZE() > 0);
  }
}
