//! Global constants and (environment) variables.

#![allow(non_snake_case)]

use std::env;
use std::sync::OnceLock;

/// Mutex locking timeout, by default [`u64::MAX`].
///
/// NOTE: This constants can be configured through `RSVIM_MUTEX_TIMEOUT` environment variable.
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

/// Buffer size of IO operations such as file, sockets, etc.
pub fn IO_BUF_SIZE() -> usize {
  8192_usize
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
