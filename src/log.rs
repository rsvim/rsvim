use crate::cli::Cli;
use std::io;
use tracing::{self, debug, Level};
use tracing_appender;
use tracing_subscriber::{self, fmt, EnvFilter};

enum LogMode {
  ERROR,
  INFO,
  DEBUG,
}

pub fn init(cli: &Cli) {
  let mut mode = LogMode::ERROR;
  if cli.debug() {
    mode = LogMode::DEBUG;
  } else if cli.verbose() {
    mode = LogMode::INFO;
  }

  match mode {
    LogMode::DEBUG => {
      let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_level(true)
        .with_ansi(false)
        .with_env_filter(EnvFilter::from_default_env())
        .with_max_level(Level::TRACE)
        .with_writer(tracing_appender::rolling::never(".", "rsvim.log"))
        .with_writer(io::stderr)
        .finish();
      tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to initialize tracing log");
    }
    LogMode::INFO => {
      let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_level(true)
        .with_ansi(false)
        .with_env_filter(EnvFilter::from_default_env())
        .with_max_level(Level::INFO)
        .with_writer(io::stderr)
        .finish();
      tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to initialize tracing log");
    }
    _ => {
      let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_level(true)
        .with_ansi(false)
        .with_env_filter(EnvFilter::from_default_env())
        .with_max_level(Level::ERROR)
        .with_writer(io::stderr)
        .finish();
      tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to initialize tracing log");
    }
  };
}
