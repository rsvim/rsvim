//! Command line options.

use crate::flags_impl;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

flags_impl!(
  SpecialFlags,
  u8,
  VERSION,
  0b0000_0001,
  SHORT_HELP,
  0b0000_0010,
  LONG_HELP,
  0b0000_0100
);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliSpecialOptions {
  // version
  // short_help
  // long_help
  flags: SpecialFlags,
}

impl CliSpecialOptions {
  pub fn new(version: bool, short_help: bool, long_help: bool) -> Self {
    let mut flags = SpecialFlags::empty();
    flags.set(SpecialFlags::VERSION, version);
    flags.set(SpecialFlags::SHORT_HELP, short_help);
    flags.set(SpecialFlags::LONG_HELP, long_help);
    Self { flags }
  }

  pub fn version(&self) -> bool {
    self.flags.contains(SpecialFlags::VERSION)
  }

  pub fn short_help(&self) -> bool {
    self.flags.contains(SpecialFlags::SHORT_HELP)
  }

  pub fn long_help(&self) -> bool {
    self.flags.contains(SpecialFlags::LONG_HELP)
  }

  #[cfg(test)]
  pub fn empty() -> Self {
    Self {
      flags: SpecialFlags::empty(),
    }
  }
}

#[derive(Debug, Clone)]
/// Command line options.
pub struct CliOptions {
  // Special opts
  special_opts: CliSpecialOptions,

  // Normal opts
  file: Vec<PathBuf>,
  headless: bool,
}

fn parse(mut parser: lexopt::Parser) -> Result<CliOptions, lexopt::Error> {
  use lexopt::prelude::*;

  let mut version: bool = false;
  let mut short_help: bool = false;
  let mut long_help: bool = false;
  let mut headless: bool = false;
  let mut file: Vec<PathBuf> = vec![];

  while let Some(arg) = parser.next()? {
    match arg {
      Short('h') => {
        short_help = true;
      }
      Long("help") => {
        long_help = true;
      }
      Short('V') | Long("version") => {
        version = true;
      }
      Long("headless") => {
        headless = true;
      }
      Value(filename) => {
        file.push(Path::new(&filename).to_path_buf());
      }
      _ => return Err(arg.unexpected()),
    }
  }

  Ok(CliOptions {
    special_opts: CliSpecialOptions::new(version, short_help, long_help),
    file,
    headless,
  })
}

impl CliOptions {
  pub fn from_env() -> Result<Self, lexopt::Error> {
    parse(lexopt::Parser::from_env())
  }

  pub fn from_args<I>(args: I) -> Result<Self, lexopt::Error>
  where
    I: IntoIterator,
    I::Item: Into<OsString>,
  {
    parse(lexopt::Parser::from_args(args))
  }

  pub fn from_string_args(
    args: &Vec<std::ffi::OsString>,
  ) -> Result<Self, lexopt::Error> {
    parse(lexopt::Parser::from_args(args))
  }

  /// Special options.
  pub fn special_opts(&self) -> &CliSpecialOptions {
    &self.special_opts
  }

  /// Input files.
  pub fn file(&self) -> &Vec<PathBuf> {
    &self.file
  }

  /// Headless mode.
  pub fn headless(&self) -> bool {
    self.headless
  }

  #[cfg(test)]
  pub fn new(
    special_opts: CliSpecialOptions,
    file: Vec<PathBuf>,
    headless: bool,
  ) -> Self {
    Self {
      special_opts,
      file,
      headless,
    }
  }

  #[cfg(test)]
  pub fn empty() -> Self {
    Self {
      special_opts: CliSpecialOptions::empty(),
      file: vec![],
      headless: true,
    }
  }
}
