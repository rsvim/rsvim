use crate::cli::Cli;
use std::io::{self, Stderr, StderrLock, Stdout, StdoutLock};
use tracing::{self, debug, Level};
use tracing_appender;
use tracing_core::{Level, Metadata};
use tracing_subscriber;
use tracing_subscriber::fmt::writer::MakeWriter;

struct ConsoleAppender {
  stdout: Stdout,
  stderr: Stderr,
}

enum ConsoleAppendLock<'a> {
  Stdout(StdoutLock<'a>),
  Stderr(StderrLock<'a>),
}

impl<'a> io::Write for ConsoleAppendLock<'a> {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    match self {
      ConsoleAppendLock::Stdout(lock) => lock.write(buf),
      ConsoleAppendLock::Stderr(lock) => lock.write(buf),
    }
  }

  fn write_all(&mut self, mut buf: &[u8]) -> io::Result<()> {
    match self {
      ConsoleAppendLock::Stdout(lock) => lock.write_all(buf),
      ConsoleAppendLock::Stderr(lock) => lock.write_all(buf),
    }
  }

  fn flush(&mut self) -> io::Result<()> {
    match self {
      ConsoleAppendLock::Stdout(lock) => lock.flush(),
      ConsoleAppendLock::Stderr(lock) => lock.flush(),
    }
  }
}

impl<'a> MakeWriter<'a> for ConsoleAppender {
  type Writer = ConsoleAppendLock<'a>;

  fn make_writer(&'a self) -> Self::Writer {
    ConsoleAppendLock::Stdout(self.stdout.lock())
  }

  fn make_writer_for(&'a self, meta: &Metadata<'_>) -> Self::Writer {
    if meta.level() <= &Level::WARN {
      ConsoleAppendLock::Stderr(self.stderr.lock())
    } else {
      ConsoleAppendLock::Stdout(self.stdout.lock())
    }
  }
}

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

  let mut subscriber = tracing_subscriber::FmtSubscriber::builder()
    .with_file(true)
    .with_line_number(true)
    .with_thread_ids(true)
    .with_thread_names(true)
    .with_level(true)
    .with_ansi(false)
    .with_max_level(log_level);

  if use_file_appender {
    let file_appender = tracing_appender::rolling::never(".", "rsvim.log");
    subscriber = subscriber.with_writer(file_appender);
  }
  if use_console_appender {
    let console_appender = ConsoleAppender {
      stdout: io::stdout(),
      stderr: io::stderr(),
    };
    subscriber = subscriber.with_writer(console_appender);
  }
  let subscriber = subscriber.finish();
  tracing::subscriber::set_global_default(subscriber).expect("Failed to initialize tracing log");
  debug!("Initialize tracing log");
}
