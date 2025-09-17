//! Loggings for testing.
//!
//! NOTE: This module should only be used in unit tests, not some where else.

use crate::constant::RSVIM_LOG;
use std::sync::Once;

const RUST_LOG: &str = "RUST_LOG";

/// Initialize the logging prints to `stderr`.
pub fn init() {
  static INITIALIZED: Once = Once::new();
  INITIALIZED.call_once(|| {
    let filter = env_filter::Builder::from_env(RSVIM_LOG).build();
    let formatter = "%T%.3f";

    fern::Dispatch::new()
      .filter(move |metadata| filter.enabled(metadata))
      .format(|out, message, record| {
        out.finish(format_args!(
          "{} {:<5} {}:{}| {}",
          jiff::Zoned::now().strftime(formatter),
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
