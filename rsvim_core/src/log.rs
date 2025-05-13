//! Logging utils.

use jiff::Zoned;
use tracing;
use tracing_appender;
use tracing_subscriber::{self, EnvFilter};

/// Initialize logging.
///
/// It uses `RSVIM_LOG` environment variable to control the logging level.
/// Defaults to `error`.
pub fn init() {
  let env_filter = EnvFilter::try_from_env("RSVIM_LOG").unwrap_or(EnvFilter::from_str("error"));

  if env_filter.max_level_hint().is_some()
    && env_filter.max_level_hint().unwrap() >= tracing::Level::TRACE
  {
    // If trace/debug log is enabled, write logs into file.
    let now = Zoned::now();
    let log_name = format!(
      "rsvim-{:0>4}{:0>2}{:0>2}-{:0>2}{:0>2}{:0>2}-{:0>3}.log",
      now.date().year(),
      now.date().month(),
      now.date().day(),
      now.time().hour(),
      now.time().minute(),
      now.time().second(),
      now.time().millisecond(),
    );
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
      .with_file(true)
      .with_line_number(true)
      .with_thread_ids(true)
      .with_thread_names(true)
      .with_level(true)
      .with_ansi(false)
      .with_env_filter(env_filter)
      .with_writer(tracing_appender::rolling::never(".", log_name))
      .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
  } else {
    // If trace/debug log is disabled, write logs into stderr.
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
      .with_file(true)
      .with_line_number(true)
      .with_thread_ids(true)
      .with_thread_names(true)
      .with_level(true)
      .with_ansi(false)
      .with_env_filter(env_filter)
      .with_writer(std::io::stderr)
      .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
  }
}
