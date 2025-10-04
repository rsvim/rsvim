//! Command line options.

use bitflags::bitflags;
use std::ffi::OsString;
use std::fmt::Debug;
use std::path::Path;
use std::path::PathBuf;

bitflags! {
  #[derive(Copy, Clone, PartialEq, Eq)]
  struct SpecialFlags : u8{
    const VERSION = 1;
    const SHORT_HELP = 1 << 1;
    const LONG_HELP = 1 << 2;
  }
}

impl Debug for SpecialFlags {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("SpecialFlags")
      .field("bits", &format!("{:b}", self.bits()))
      .finish()
  }
}

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
    if version {
      flags.insert(SpecialFlags::VERSION);
    }
    if short_help {
      flags.insert(SpecialFlags::SHORT_HELP);
    }
    if long_help {
      flags.insert(SpecialFlags::LONG_HELP);
    }
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

bitflags! {
  #[derive(Copy, Clone)]
  struct Flags: u8 {
    const HEADLESS = 1;
  }
}

impl Debug for Flags {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Flags")
      .field("bits", &format!("{:b}", self.bits()))
      .finish()
  }
}

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

  let mut version: bool = false;
  let mut short_help: bool = false;
  let mut long_help: bool = false;
  let mut flags: Flags = Flags::empty();
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
        flags.insert(Flags::HEADLESS);
      }
      Value(filename) => {
        file.push(Path::new(&filename).to_path_buf());
      }
      _ => return Err(arg.unexpected()),
    }
  }

  Ok(CliOptions {
    special_opts: CliSpecialOptions::new(version, short_help, long_help),
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
    let mut flags: Flags = Flags::empty();
    if headless {
      flags.insert(Flags::HEADLESS);
    }
    Self {
      special_opts,
      flags,
      file,
    }
  }

  #[cfg(test)]
  pub fn empty() -> Self {
    let flags: Flags = Flags::HEADLESS;
    Self {
      special_opts: CliSpecialOptions::empty(),
      flags,
      file: vec![],
    }
  }
}
