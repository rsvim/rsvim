//! Testing logging.

use tracing;
use tracing_subscriber;

#[cfg(test)]
pub fn init() {
  let subscriber = tracing_subscriber::FmtSubscriber::builder()
    .with_file(true)
    .with_line_number(true)
    .with_thread_ids(true)
    .with_thread_names(true)
    .with_level(true)
    .with_ansi(true)
    .with_max_level(tracing::Level::TRACE)
    .with_writer(std::io::stderr)
    .finish();
  tracing::subscriber::set_global_default(subscriber).unwrap();
}
