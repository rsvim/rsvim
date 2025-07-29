//! Global constants.

use once_cell::sync::Lazy;
use regex::Regex;
use std::time::Duration;

use path_config::*;

pub mod path_config;

#[cfg(test)]
mod path_config_tests;

/// Mutex locking timeout in seconds, by default is [`u64::MAX`].
///
/// NOTE: This constant can be configured through `RSVIM_MUTEX_TIMEOUT_SECS` environment variable.
pub static MUTEX_TIMEOUT_SECS: Lazy<u64> = Lazy::new(|| {
  std::env::var("RSVIM_MUTEX_TIMEOUT_SECS")
    .map(|v| v.parse::<u64>().unwrap_or(u64::MAX))
    .unwrap_or(u64::MAX)
});

/// Mutex locking timeout duration, by default is [`u64::MAX`] seconds.
pub static MUTEX_TIMEOUT: Lazy<Duration> =
  Lazy::new(|| Duration::from_secs(*MUTEX_TIMEOUT_SECS));

/// Buffer size for channels communication, by default is 1000.
///
/// NOTE: This constant can be configured through `RSVIM_CHANNEL_BUF_SIZE` environment variable.
pub static CHANNEL_BUF_SIZE: Lazy<usize> = Lazy::new(|| {
  std::env::var("RSVIM_CHANNEL_BUF_SIZE")
    .map(|v| v.parse::<usize>().unwrap_or(1000_usize))
    .unwrap_or(1000_usize)
});

/// Windows drive's full path beginning regex, for example full file path begins with `C:\\`.
pub static WINDOWS_DRIVE_BEGIN_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^[a-zA-Z]:\\").unwrap());

/// Http(s) url beginning regex, for example url begins with `http(s)?://`.
pub static HTTP_URL_BEGIN_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^(http|https)://").unwrap());

/// Global path configuration
pub static PATH_CONFIG: Lazy<PathConfig> = Lazy::new(PathConfig::new);
