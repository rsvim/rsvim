//! Logging utils.

use time::{format_description, Date, Month, OffsetDateTime, Time, UtcOffset};
use tracing;
use tracing_appender;
use tracing_subscriber::{self, EnvFilter};
use tzdb;

/// Initialize logging.
///
/// It uses `RUST_LOG` environment variable to control the logging level.
/// Defaults to `INFO`.
pub fn init() {
  let env_filter = EnvFilter::from_default_env();

  if env_filter.max_level_hint().is_some()
    && env_filter.max_level_hint().unwrap() >= tracing::Level::DEBUG
  {
    // If debug log is enabled, write logs into file.
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
      .with_env_filter(env_filter)
      .with_writer(tracing_appender::rolling::never(".", log_name))
      .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
  } else {
    // If debug log is disabled, write logs into stderr.
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
