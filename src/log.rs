pub mod console_make_writer;

use crate::cli;
use time::{format_description, Date, Month, OffsetDateTime, Time, UtcOffset};
use tracing;
use tracing_appender;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{self, EnvFilter};
use tzdb;

pub fn init(c: &cli::Cli) {
  if c.debug() {
    let now = tzdb::now::local().unwrap();
    let now = OffsetDateTime::new_in_offset(
      Date::from_calendar_date(
        now.year(),
        Month::try_from(now.month()).unwrap(),
        now.month_day(),
      )
      .unwrap(),
      Time::from_hms_nano(now.hour(), now.minute(), now.second(), now.nanoseconds()).unwrap(),
      UtcOffset::from_whole_seconds(now.local_time_type().ut_offset()).unwrap(),
    );
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
      .with_max_level(tracing::Level::TRACE)
      .with_writer(console_make_writer::ConsoleMakeWriter::new())
      .finish()
      .with(
        tracing_subscriber::fmt::Layer::default()
          .with_writer(tracing_appender::rolling::never(".", log_name)),
      );
    tracing::subscriber::set_global_default(subscriber).unwrap();
  } else {
    let log_level = if c.verbose() {
      tracing::Level::INFO
    } else {
      tracing::Level::ERROR
    };
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
      .with_file(true)
      .with_line_number(true)
      .with_level(true)
      .with_env_filter(EnvFilter::from_default_env())
      .with_max_level(log_level)
      .with_writer(console_make_writer::ConsoleMakeWriter::new())
      .pretty()
      .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
  }
}
