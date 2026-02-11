//! Global constants.

// use regex::Regex;
use crate::cfg::path_cfg::PathConfig;
use once_cell::sync::Lazy;
use std::time::Duration;

pub const RSVIM_LOG: &str = "RSVIM_LOG";
pub const RSVIM_MUTEX_TIMEOUT_SECS: &str = "RSVIM_MUTEX_TIMEOUT_SECS";

/// Mutex locking timeout in seconds, by default is [`u64::MAX`].
///
/// NOTE: This constant can be configured through `RSVIM_MUTEX_TIMEOUT_SECS`
/// environment variable.
pub static MUTEX_TIMEOUT_SECS: Lazy<u64> = Lazy::new(|| {
  let default_timeout_secs = u64::MAX;
  std::env::var(RSVIM_MUTEX_TIMEOUT_SECS)
    .map(|v| v.parse::<u64>().unwrap_or(default_timeout_secs))
    .unwrap_or(default_timeout_secs)
});

/// Mutex locking timeout duration, by default is [`u64::MAX`] seconds.
pub static MUTEX_TIMEOUT: Lazy<Duration> =
  Lazy::new(|| Duration::from_secs(*MUTEX_TIMEOUT_SECS));

// /// Windows drive's full path beginning regex, for example full file path begins with `C:\\`.
// pub static WINDOWS_DRIVE_BEGIN_REGEX: Lazy<Regex> =
//   Lazy::new(|| Regex::new(r"^[a-zA-Z]:\\").unwrap());
//
// /// Http(s) url beginning regex, for example url begins with `http(s)?://`.
// pub static HTTP_URL_BEGIN_REGEX: Lazy<Regex> =
//   Lazy::new(|| Regex::new(r"^(http|https)://").unwrap());

pub static PATH_CONFIG: Lazy<PathConfig> = Lazy::new(PathConfig::new);
