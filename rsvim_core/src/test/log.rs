//! Loggings for testing.
//!
//! NOTE: This module should only be used in unit tests, not some where else.

use crate::log::RSVIM_LOG;

/// Initialize the logging prints to `stderr`.
pub fn init() {
  use std::sync::Once;

  static INITIALIZED: Once = Once::new();
  INITIALIZED.call_once(|| {
    let filter = env_filter::Builder::from_env(RSVIM_LOG).build();

    fern::Dispatch::new()
      .filter(move |metadata| filter.enabled(metadata))
      .format(|out, message, record| {
        out.finish(format_args!(
          "{:<5} {}:{}| {}",
          record.level(),
          record.target(),
          record.line().unwrap_or(0),
          message
        ))
      })
      .chain(std::io::stdout())
      .apply()
      .unwrap();
  });
}
