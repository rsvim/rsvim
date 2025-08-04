//! Loggings for testing.
//!
//! NOTE: This module should only be used in unit tests, not some where else.

use crate::log::FORMATTER;

use env_filter::Builder;
use jiff::Zoned;

/// Initialize the logging prints to `stderr`.
pub fn init() {
  use std::sync::Once;

  static INITIALIZED: Once = Once::new();
  INITIALIZED.call_once(|| {
    let env_filter = Builder::from_env("RSVIM_LOG").build();

    fern::Dispatch::new()
      .filter(move |metadata| env_filter.enabled(metadata))
      .format(|out, message, record| {
        out.finish(format_args!(
          "[{} {} {}:{}] {}",
          Zoned::now().strftime(FORMATTER),
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
