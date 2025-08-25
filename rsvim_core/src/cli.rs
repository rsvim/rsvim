//! Command line options.

use crate::js::v8_version;

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliSpecialOptions {
  version: bool,
  short_help: bool,
  long_help: bool,
}

impl CliSpecialOptions {
  pub fn new(version: bool, short_help: bool, long_help: bool) -> Self {
    Self {
      version,
      short_help,
      long_help,
    }
  }

  pub fn version(&self) -> bool {
    self.version
  }

  pub fn short_help(&self) -> bool {
    self.short_help
  }

  pub fn long_help(&self) -> bool {
    self.long_help
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

#[derive(Debug, Clone)]
/// Command line options.
pub struct CliOptions {
  // Special opts
  special_opts: CliSpecialOptions,

  // Normal opts
  file: Vec<PathBuf>,
  headless: bool,
}

pub const RSVIM_BIN_NAME: &str = "RSVIM_BIN_NAME";
pub const RSVIM_PKG_VERSION: &str = "RSVIM_PKG_VERSION";
pub const RSVIM_V8_VERSION: &str = "RSVIM_V8_VERSION";

// --headless (experimental)  Run in headless mode without TUI
pub const SHORT_HELP: &str =
  include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/CLI_SHORT_HELP.TXT"));

// --headless (experimental)
//     Run in headless mode without TUI. In this mode, rsvim doesn't enter
//     terminal's raw mode, it uses STDIN to receive javascript script, and
//     uses STDOUT, STDERR to print messages instead of rendering TUI. All
//     internal data structures (such as buffers, windows, command-line,
//     etc) and scripts/plugins will still be initialized
pub const LONG_HELP: &str =
  include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/CLI_LONG_HELP.TXT"));

pub const VERSION: &str =
  "{RSVIM_BIN_NAME} {RSVIM_PKG_VERSION} (v8 {RSVIM_V8_VERSION})";

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
}
