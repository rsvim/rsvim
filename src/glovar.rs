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

/// Fixed rate update intervals (in milliseconds), defaults to 10 (i.e. 100/second).
///
/// NOTE: This constant can be configured through `RSVIM_FIXED_RATE_UPDATE_MILLIS` environment variable.
pub fn FIXED_RATE_UPDATE_MILLIS() -> u64 {
  static VALUE: OnceLock<u64> = OnceLock::new();

  *VALUE.get_or_init(|| match env::var("RSVIM_FIXED_RATE_UPDATE_MILLIS") {
    Ok(v1) => match v1.parse::<u64>() {
      Ok(v2) => v2,
      _ => 10_u64,
    },
    _ => 10_u64,
  })
}

static PATH_CONFIG_VALUE: OnceLock<PathConfig> = OnceLock::new();

/// Config file path, the config file is located by following orders:
///
/// 1. $XDG_CONFIG_HOME/rsvim/rsvim.{ts,js}
/// 2. $HOME/.rsvim/rsvim.{ts.js}
/// 3. $HOME/.rsvim.{ts.js}
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

/// Cache directory path.
pub fn CACHE_DIR_PATH() -> PathBuf {
  PATH_CONFIG_VALUE
    .get_or_init(PathConfig::new)
    .cache_dir()
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
