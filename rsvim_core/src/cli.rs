//! Command line options.

use crate::flags_impl;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

flags_impl!(
  SpecialFlags,
  u8,
  VERSION,
  1,
  SHORT_HELP,
  1 << 1,
  LONG_HELP,
  1 << 2,
);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliSpecialOptions {
  // version
  // short_help
  // long_help
  flags: SpecialFlags,
}

impl CliSpecialOptions {
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
  pub fn new(version: bool, short_help: bool, long_help: bool) -> Self {
    let mut flags = SpecialFlags::empty();
    flags.set(SpecialFlags::VERSION, version);
    flags.set(SpecialFlags::SHORT_HELP, short_help);
    flags.set(SpecialFlags::LONG_HELP, long_help);
    Self { flags }
  }

  #[cfg(test)]
  pub fn empty() -> Self {
    Self {
      flags: SpecialFlags::empty(),
    }
  }
}

flags_impl!(Flags, u8, HEADLESS, 0b0000_0001);

#[derive(Debug, Clone)]
/// Command line options.
pub struct CliOptions {
  // Special opts
  special_opts: CliSpecialOptions,

  // headless
  flags: Flags,

  // Normal opts
  file: Vec<PathBuf>,
}

fn parse(mut parser: lexopt::Parser) -> Result<CliOptions, lexopt::Error> {
  use lexopt::prelude::*;

  // let mut version: bool = false;
  // let mut short_help: bool = false;
  // let mut long_help: bool = false;
  let mut special_flags: SpecialFlags = SpecialFlags::empty();
  let mut flags: Flags = Flags::empty();
  let mut file: Vec<PathBuf> = vec![];

  while let Some(arg) = parser.next()? {
    match arg {
      Short('h') => {
        special_flags.insert(SpecialFlags::SHORT_HELP);
      }
      Long("help") => {
        special_flags.insert(SpecialFlags::LONG_HELP);
      }
      Short('V') | Long("version") => {
        special_flags.insert(SpecialFlags::VERSION);
      }
      Long("headless") => {
        flags.insert(Flags::HEADLESS);
      }
      Value(filename) => {
        file.push(Path::new(&filename).to_path_buf());
      }
      _ => return Err(arg.unexpected()),
    }
  }

  Ok(CliOptions {
    special_opts: CliSpecialOptions {
      flags: special_flags,
    },
    flags,
    file,
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
    self.flags.contains(Flags::HEADLESS)
  }

  #[cfg(test)]
  pub fn new(
    special_opts: CliSpecialOptions,
    file: Vec<PathBuf>,
    headless: bool,
  ) -> Self {
    let mut flags = Flags::empty();
    flags.set(Flags::HEADLESS, headless);
    Self {
      special_opts,
      flags,
      file,
    }
  }

  #[cfg(test)]
  pub fn empty() -> Self {
    Self {
      special_opts: CliSpecialOptions::empty(),
      // headless=true
      flags: Flags::HEADLESS,
      file: vec![],
    }
  }
}
