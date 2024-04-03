use crate::cli::Cli;
use std::io;
use tracing::{self, debug, Level};
use tracing_appender;
use tracing_subscriber::{self, fmt, EnvFilter};

pub fn init(cli: &Cli) {
  let mut log_level = Level::WARN;
  let mut use_file_appender = false;
  let mut use_console_appender = true;
  if cli.debug() {
    log_level = Level::DEBUG;
    use_file_appender = true;
    use_console_appender = true;
  } else if cli.verbose() {
    log_level = Level::INFO;
    use_console_appender = true;
  }

  let subscriber = tracing_subscriber::FmtSubscriber::builder()
    .with_file(true)
    .with_line_number(true)
    .with_thread_ids(true)
    .with_thread_names(true)
    .with_level(true)
    .with_ansi(false)
    .with_env_filter(EnvFilter::from_default_env())
    .with_max_level(log_level);
  if use_file_appender {
    subscriber.with_writer(tracing_appender::rolling::never(".", "rsvim.log"));
  }
  if use_console_appender {
    subscriber.with_writer(io::stderr);
  }
  let subscriber = subscriber.finish();
  tracing::subscriber::set_global_default(subscriber).expect("Failed to initialize tracing log");
  debug!("Initialize tracing log");
}
