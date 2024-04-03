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
  let mut use_file_appender = false;
  let mut use_console_appender = true;
  if cli.debug() {
    use_file_appender = true;
    use_console_appender = true;
  } else if cli.verbose() {
    use_console_appender = true;
  }

  let file_appender = match use_file_appender {
    true => Some(tracing_appender::rolling::never(".", "rsvim.log")),
    _ => None,
  };
  let console_appender = match use_console_appender {
    true => Some(ConsoleAppender),
    _ => None,
  };

  let subscriber = tracing_subscriber::FmtSubscriber::builder()
    .with_file(true)
    .with_line_number(true)
    .with_thread_ids(true)
    .with_thread_names(true)
    .with_level(true)
    .with_ansi(false)
    .with_max_level(log_level);
  tracing::subscriber::set_global_default(subscriber).expect("Failed to initialize tracing log");
  debug!("Initialize tracing log");
}
