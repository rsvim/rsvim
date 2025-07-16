//! Global constants.

use regex::Regex;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::time::Duration;

use path_config::PathConfig;

pub mod path_config;

#[cfg(test)]
mod path_config_tests;

/// Mutex locking timeout in seconds, by default is [`u64::MAX`].
///
/// NOTE: This constant can be configured through `RSVIM_MUTEX_TIMEOUT_SECS` environment variable.
pub static MUTEX_TIMEOUT_SECS: LazyLock<u64> = LazyLock::new(|| {
  std::env::var("RSVIM_MUTEX_TIMEOUT_SECS")
    .map(|v| v.parse::<u64>().unwrap_or(u64::MAX))
    .unwrap_or(u64::MAX)
});

/// Mutex locking timeout duration, by default is [`u64::MAX`] seconds.
pub static MUTEX_TIMEOUT: LazyLock<Duration> =
  LazyLock::new(|| Duration::from_secs(*MUTEX_TIMEOUT_SECS));

/// Buffer size for channels communication, by default is 1000.
///
/// NOTE: This constant can be configured through `RSVIM_CHANNEL_BUF_SIZE` environment variable.
pub static CHANNEL_BUF_SIZE: LazyLock<usize> = LazyLock::new(|| {
  std::env::var("RSVIM_CHANNEL_BUF_SIZE")
    .map(|v| v.parse::<usize>().unwrap_or(1000_usize))
    .unwrap_or(1000_usize)
});

static PATH_CONFIG: LazyLock<PathConfig> = LazyLock::new(PathConfig::new);

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
pub static CONFIG_ENTRY_PATH: LazyLock<Option<PathBuf>> =
  LazyLock::new(|| PATH_CONFIG.config_entry().clone());

/// User config home directory, it can be either one of following directories:
///
/// 1. `$XDG_CONFIG_HOME/rsvim/` or `$HOME/.config/rsvim/`.
/// 2. `$HOME/.rsvim/`
pub static CONFIG_HOME_PATH: LazyLock<Option<PathBuf>> =
  LazyLock::new(|| PATH_CONFIG.config_home().clone());

/// Cache home directory, i.e. `$XDG_CACHE_HOME/rsvim`.
pub static CACHE_HOME_PATH: LazyLock<PathBuf> = LazyLock::new(|| PATH_CONFIG.cache_home().clone());

/// Data home directory, i.e. `$XDG_DATA_HOME/rsvim`.
pub static DATA_HOME_PATH: LazyLock<PathBuf> = LazyLock::new(|| PATH_CONFIG.data_home().clone());

/// Windows drive's full path detect regex, for example full file path begins with `C:\\`.
pub static WINDOWS_DRIVE_REGEX: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^[a-zA-Z]:\\").unwrap());

/// Http(s) url detect regex, for example url begins with `http(s)?://`.
pub static HTTP_URL_REGEX: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^(http|https)://").unwrap());
