use crate::cli::Cli;
use std::io;
use time::{format_description, OffsetDateTime};
use tracing::{self, Level};
use tracing_appender;
use tracing_subscriber::{self, EnvFilter};

pub fn init(cli: &Cli) {
  if cli.debug() {
    let now = OffsetDateTime::now_local().unwrap();
    let fmt = format_description::parse(
      "rsvim-[year][month][day]-[hour][minute][second]-[subsecond digits:3].log",
    )
    .unwrap();
    let log_name = now.format(&fmt).unwrap();
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
      .with_file(true)
      .with_line_number(true)
      .with_thread_ids(true)
      .with_thread_names(true)
      .with_level(true)
      .with_ansi(false)
      .with_env_filter(EnvFilter::from_default_env())
      .with_max_level(Level::TRACE)
      .with_writer(tracing_appender::rolling::never(".", log_name))
      .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
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
    tracing::subscriber::set_global_default(subscriber).unwrap();
  }
}
