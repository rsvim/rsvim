use crate::cli::Cli;
use chrono::prelude::Local;
use std::io;
use tracing::{self, Level};
use tracing_appender;
use tracing_subscriber::{self, EnvFilter};

pub fn init(cli: &Cli) {
  if cli.debug() {
    let now = Local::now();
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
      .with_file(true)
      .with_line_number(true)
      .with_thread_ids(true)
      .with_thread_names(true)
      .with_level(true)
      .with_ansi(false)
      .with_env_filter(EnvFilter::from_default_env())
      .with_max_level(Level::TRACE)
      .with_writer(tracing_appender::rolling::never(
        ".",
        format!(
          "rsvim-{}.log",
          now.format("%Y-%m-%d-%H-%M-%S-%3f").to_string()
        ),
      ))
      .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to initialize tracing log");
  } else {
    let log_level = match cli.verbose() {
      true => Level::INFO,
      _ => Level::ERROR,
    };
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
      .with_file(true)
      .with_line_number(true)
      .with_thread_ids(true)
      .with_thread_names(true)
      .with_level(true)
      .with_env_filter(EnvFilter::from_default_env())
      .with_max_level(log_level)
      .with_writer(io::stderr)
      .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to initialize tracing log");
  }
}
