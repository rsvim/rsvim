//! Global constants.

// use regex::Regex;
use std::sync::LazyLock;
use std::time::Duration;

pub const RSVIM_LOG: &str = "RSVIM_LOG";
pub const RSVIM_MUTEX_TIMEOUT_SECS: &str = "RSVIM_MUTEX_TIMEOUT_SECS";
pub const RSVIM_CHANNEL_BUF_SIZE: &str = "RSVIM_CHANNEL_BUF_SIZE";

/// Mutex locking timeout in seconds, by default is [`u64::MAX`].
///
/// NOTE: This constant can be configured through `RSVIM_MUTEX_TIMEOUT_SECS`
/// environment variable.
pub static MUTEX_TIMEOUT_SECS: LazyLock<u64> = LazyLock::new(|| {
  let default_timeout_secs = u64::MAX;
  std::env::var(RSVIM_MUTEX_TIMEOUT_SECS)
    .map(|v| v.parse::<u64>().unwrap_or(default_timeout_secs))
    .unwrap_or(default_timeout_secs)
});

/// Mutex locking timeout duration, by default is [`u64::MAX`] seconds.
pub static MUTEX_TIMEOUT: LazyLock<Duration> =
  LazyLock::new(|| Duration::from_secs(*MUTEX_TIMEOUT_SECS));

/// Buffer size for channels communication, by default is 1000.
///
/// NOTE: This constant can be configured through `RSVIM_CHANNEL_BUF_SIZE`
/// environment variable.
pub static CHANNEL_BUF_SIZE: LazyLock<usize> = LazyLock::new(|| {
  let default_buf_size = 100_usize;
  std::env::var(RSVIM_CHANNEL_BUF_SIZE)
    .map(|v| v.parse::<usize>().unwrap_or(default_buf_size))
    .unwrap_or(default_buf_size)
});

// /// Windows drive's full path beginning regex, for example full file path begins with `C:\\`.
// pub static WINDOWS_DRIVE_BEGIN_REGEX: LazyLock<Regex> =
//   LazyLock::new(|| Regex::new(r"^[a-zA-Z]:\\").unwrap());
//
// /// Http(s) url beginning regex, for example url begins with `http(s)?://`.
// pub static HTTP_URL_BEGIN_REGEX: LazyLock<Regex> =
//   LazyLock::new(|| Regex::new(r"^(http|https)://").unwrap());
