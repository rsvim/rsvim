use crate::cli::Cli;
use std::io::{self, Stderr, StderrLock, Stdout, StdoutLock};
use time::{format_description, Date, Month, OffsetDateTime, Time, UtcOffset};
use tracing::{self, Level};
use tracing_appender;
use tracing_core;
use tracing_subscriber::fmt::writer::MakeWriter;
use tracing_subscriber::{self, EnvFilter};
use tzdb;

struct ConsoleMakeWriter {
  stdout: Stdout,
  stderr: Stderr,
}

enum ConsoleLock<'a> {
  Stdout(StdoutLock<'a>),
  Stderr(StderrLock<'a>),
}

impl<'a> io::Write for ConsoleLock<'a> {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    match self {
      ConsoleLock::Stdout(lock) => lock.write(buf),
      ConsoleLock::Stderr(lock) => lock.write(buf),
    }
  }

  fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
    match self {
      ConsoleLock::Stdout(lock) => lock.write_all(buf),
      ConsoleLock::Stderr(lock) => lock.write_all(buf),
    }
  }

  fn flush(&mut self) -> io::Result<()> {
    match self {
      ConsoleLock::Stdout(lock) => lock.flush(),
      ConsoleLock::Stderr(lock) => lock.flush(),
    }
  }
}

impl<'a> MakeWriter<'a> for ConsoleMakeWriter {
  type Writer = ConsoleLock<'a>;

  fn make_writer(&'a self) -> Self::Writer {
    ConsoleLock::Stdout(self.stdout.lock())
  }

  fn make_writer_for(&'a self, meta: &tracing_core::Metadata<'_>) -> Self::Writer {
    if meta.level() <= &tracing_core::Level::WARN {
      ConsoleLock::Stderr(self.stderr.lock())
    } else {
      ConsoleLock::Stdout(self.stdout.lock())
    }
  }
}

struct DebugConsoleMakeWriter {
  stdout: Stdout,
  stderr: Stderr,
}

enum DebugConsoleLock<'a> {
  Stdout(StdoutLock<'a>),
  Stderr(StderrLock<'a>),
}

impl<'a> io::Write for DebugConsoleLock<'a> {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    match self {
      DebugConsoleLock::Stdout(_) => Ok(0),
      DebugConsoleLock::Stderr(lock) => lock.write(buf),
    }
  }

  fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
    match self {
      DebugConsoleLock::Stdout(_) => Ok(()),
      DebugConsoleLock::Stderr(lock) => lock.write_all(buf),
    }
  }

  fn flush(&mut self) -> io::Result<()> {
    match self {
      DebugConsoleLock::Stdout(_) => Ok(()),
      DebugConsoleLock::Stderr(lock) => lock.flush(),
    }
  }
}

impl<'a> MakeWriter<'a> for DebugConsoleMakeWriter {
  type Writer = DebugConsoleLock<'a>;

  fn make_writer(&'a self) -> Self::Writer {
    DebugConsoleLock::Stdout(self.stdout.lock())
  }

  fn make_writer_for(&'a self, meta: &tracing_core::Metadata<'_>) -> Self::Writer {
    if meta.level() <= &tracing_core::Level::WARN {
      DebugConsoleLock::Stderr(self.stderr.lock())
    } else {
      DebugConsoleLock::Stdout(self.stdout.lock())
    }
  }
}

pub fn init(cli: &Cli) {
  if cli.debug() {
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
      .with_max_level(Level::TRACE)
      .with_writer(tracing_appender::rolling::never(".", log_name))
      .with_writer(DebugConsoleMakeWriter {
        stdout: io::stdout(),
        stderr: io::stderr(),
      })
      .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
  } else {
    let log_level = if cli.verbose() {
      Level::INFO
    } else {
      Level::ERROR
    };
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
      .with_file(true)
      .with_line_number(true)
      .with_level(true)
      .with_env_filter(EnvFilter::from_default_env())
      .with_max_level(log_level)
      .with_writer(ConsoleMakeWriter {
        stdout: io::stdout(),
        stderr: io::stderr(),
      })
      .pretty()
      .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
  }
}
