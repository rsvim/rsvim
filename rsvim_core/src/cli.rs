//! Command line options.

use crate::flags_impl;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

flags_impl!(SpecialFlags, u8, VERSION, SHORT_HELP, LONG_HELP);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecialCliOptions {
  pub version: bool,
  pub short_help: bool,
  pub long_help: bool,
}

impl SpecialCliOptions {
  #[cfg(test)]
  pub fn new(version: bool, short_help: bool, long_help: bool) -> Self {
    Self {
      version,
      short_help,
      long_help,
    }
  }

  #[cfg(test)]
  pub fn empty() -> Self {
    Self {
      version: false,
      short_help: false,
      long_help: false,
    }
  }
}

flags_impl!(Flags, u8, HEADLESS);

#[derive(Debug, Clone)]
/// Command line options.
pub struct CliOptions {
  // Special opts
  special_opts: SpecialCliOptions,

  headless: bool,

  // Normal opts
  file: Vec<PathBuf>,
}

fn parse(mut parser: lexopt::Parser) -> Result<CliOptions, lexopt::Error> {
  use lexopt::prelude::*;

  let mut version: bool = false;
  let mut short_help: bool = false;
  let mut long_help: bool = false;
  let mut headless = false;
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
    special_opts: SpecialCliOptions {
      version,
      short_help,
      long_help,
    },
    headless,
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
  pub fn special_opts(&self) -> &SpecialCliOptions {
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
    special_opts: SpecialCliOptions,
    file: Vec<PathBuf>,
    headless: bool,
  ) -> Self {
    Self {
      special_opts,
      headless,
      file,
    }
  }

  #[cfg(test)]
  pub fn empty() -> Self {
    Self {
      special_opts: SpecialCliOptions::empty(),
      headless: true,
      file: vec![],
    }
  }
}
