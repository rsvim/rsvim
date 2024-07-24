//! Testing Utils.
//!
//! Note: This module should only be used in unit tests, not some where else.

#[cfg(test)]
mod log {
  pub fn init() {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
      .with_file(true)
      .with_line_number(true)
      .with_thread_ids(true)
      .with_thread_names(true)
      .with_level(true)
      .with_ansi(true)
      .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
      .with_max_level(tracing::Level::TRACE)
      .with_writer(std::io::stderr)
      .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
  }
}
