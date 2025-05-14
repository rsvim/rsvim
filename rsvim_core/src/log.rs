//! Logging utils.

use jiff::Zoned;
use tracing_appender;
use tracing_subscriber::{self, EnvFilter};

/// Initialize logging.
///
/// It uses `RSVIM_LOG` environment variable to control the logging level.
/// Defaults to `error`.
pub fn init() {
  let env_filter = EnvFilter::from_env("RSVIM_LOG");

  if env_filter.max_level_hint().unwrap() >= tracing::Level::DEBUG {
    // Only create file logs for debug level.
    let now = Zoned::now();
    let log_name = format!(
      "rsvim_{:0>4}-{:0>2}-{:0>2}_{:0>2}-{:0>2}-{:0>2}-{:0>3}.log",
      now.date().year(),
      now.date().month(),
      now.date().day(),
      now.time().hour(),
      now.time().minute(),
      now.time().second(),
      now.time().millisecond(),
    );
    tracing_subscriber::FmtSubscriber::builder()
      .with_file(true)
      .with_line_number(true)
      .with_thread_ids(true)
      .with_thread_names(true)
      .with_level(true)
      .with_ansi(false)
      .with_env_filter(env_filter)
      .with_writer(tracing_appender::rolling::never(".", log_name))
      .init();
  } else {
    // Otherwise print logs to terminal.
    tracing_subscriber::FmtSubscriber::builder()
      .with_file(true)
      .with_line_number(true)
      .with_thread_ids(false)
      .with_thread_names(false)
      .with_level(true)
      .with_ansi(false)
      .with_env_filter(env_filter)
      .with_writer(std::io::stderr)
      .init();
  }
}
