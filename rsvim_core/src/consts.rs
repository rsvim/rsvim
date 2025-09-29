//! Global constants.

// use regex::Regex;
use crate::cfg::path_cfg::PathConfig;
use std::sync::LazyLock;
use std::time::Duration;

pub const RSVIM_LOG: &str = "RSVIM_LOG";
pub const RSVIM_MUTEX_TIMEOUT_SECS: u64 = u64::MAX;
pub const RSVIM_CHANNEL_BUF_SIZE: usize = 100;

/// Mutex locking timeout in seconds, by default is [`u64::MAX`].
///
/// NOTE: This constant can be configured through `RSVIM_MUTEX_TIMEOUT_SECS`
/// environment variable.
pub static MUTEX_TIMEOUT_SECS: LazyLock<u64> = LazyLock::new(|| {
  std::env::var("RSVIM_MUTEX_TIMEOUT_SECS")
    .map(|v| v.parse::<u64>().unwrap_or(RSVIM_MUTEX_TIMEOUT_SECS))
    .unwrap_or(RSVIM_MUTEX_TIMEOUT_SECS)
});

/// Mutex locking timeout duration, by default is [`u64::MAX`] seconds.
pub static MUTEX_TIMEOUT: LazyLock<Duration> =
  LazyLock::new(|| Duration::from_secs(*MUTEX_TIMEOUT_SECS));

/// Buffer size for channels communication, by default is 1000.
///
/// NOTE: This constant can be configured through `RSVIM_CHANNEL_BUF_SIZE`
/// environment variable.
pub static CHANNEL_BUF_SIZE: LazyLock<usize> = LazyLock::new(|| {
  std::env::var("RSVIM_CHANNEL_BUF_SIZE")
    .map(|v| v.parse::<usize>().unwrap_or(RSVIM_CHANNEL_BUF_SIZE))
    .unwrap_or(RSVIM_CHANNEL_BUF_SIZE)
});

// /// Windows drive's full path beginning regex, for example full file path begins with `C:\\`.
// pub static WINDOWS_DRIVE_BEGIN_REGEX: LazyLock<Regex> =
//   LazyLock::new(|| Regex::new(r"^[a-zA-Z]:\\").unwrap());
//
// /// Http(s) url beginning regex, for example url begins with `http(s)?://`.
// pub static HTTP_URL_BEGIN_REGEX: LazyLock<Regex> =
//   LazyLock::new(|| Regex::new(r"^(http|https)://").unwrap());

pub static PATH_CONFIG: LazyLock<PathConfig> = LazyLock::new(PathConfig::new);
