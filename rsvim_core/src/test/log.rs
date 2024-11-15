//! Loggings for testing.
//!
//! NOTE: This module should only be used in unit tests, not some where else.

#[cfg(test)]
/// Initialize the logging prints to `stderr`.
pub fn init() {
  use std::sync::Once;

  static INITIALIZED: Once = Once::new();
  INITIALIZED.call_once(|| {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
      // .with_file(true)
      .with_line_number(true)
      // .with_thread_ids(true)
      // .with_thread_names(true)
      .with_level(true)
      .with_ansi(true)
      .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
      .with_writer(std::io::stderr)
      .without_time()
      .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
  });
}
