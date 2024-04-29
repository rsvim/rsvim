use crate::cli;
use time::{format_description, Date, Month, OffsetDateTime, Time, UtcOffset};
use tracing;
use tracing_appender;
use tracing_subscriber;
use tzdb;

pub fn init(c: &cli::Cli) {
  let log_level = if c.debug() {
    tracing::Level::TRACE
  } else if c.verbose() {
    tracing::Level::INFO
  } else {
    tracing::Level::ERROR
  };
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
    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    .with_max_level(log_level)
    .with_writer(tracing_appender::rolling::never(".", log_name))
    .finish();
  tracing::subscriber::set_global_default(subscriber).unwrap();
}
