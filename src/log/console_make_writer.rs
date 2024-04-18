use std::io::{self, Stderr, StderrLock};
use tracing_core;
use tracing_subscriber::fmt::writer::MakeWriter;

pub struct ConsoleMakeWriter {
  stderr: Stderr,
}

pub enum ConsoleLock<'a> {
  Stdout,
  Stderr(StderrLock<'a>),
}

impl<'a> io::Write for ConsoleLock<'a> {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    match self {
      ConsoleLock::Stdout => Ok(0),
      ConsoleLock::Stderr(lock) => lock.write(buf),
    }
  }

  fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
    match self {
      ConsoleLock::Stdout => Ok(()),
      ConsoleLock::Stderr(lock) => lock.write_all(buf),
    }
  }

  fn flush(&mut self) -> io::Result<()> {
    match self {
      ConsoleLock::Stdout => Ok(()),
      ConsoleLock::Stderr(lock) => lock.flush(),
    }
  }
}

impl<'a> MakeWriter<'a> for ConsoleMakeWriter {
  type Writer = ConsoleLock<'a>;

  fn make_writer(&'a self) -> Self::Writer {
    ConsoleLock::Stdout
  }

  fn make_writer_for(&'a self, meta: &tracing_core::Metadata<'_>) -> Self::Writer {
    if meta.level() <= &tracing_core::Level::WARN {
      ConsoleLock::Stderr(self.stderr.lock())
    } else {
      ConsoleLock::Stdout
    }
  }
}

impl ConsoleMakeWriter {
  pub fn new() -> Self {
    ConsoleMakeWriter {
      stderr: std::io::stderr(),
    }
  }
}

impl ConsoleMakeWriter {
  pub fn default() -> Self {
    Self::new()
  }
}
