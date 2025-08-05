//! Command line options.

use crate::js::v8_version;

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
/// Command line options.
pub struct CliOpt {
  file: Vec<PathBuf>,
}

const HELP: &str = r#"The VIM editor reinvented in Rust+TypeScript

Usage: rsvim [FILE]...

Arguments:
  [FILE]...  Edit file(s)

Options:
  -V, --version  Print version
  -h, --help     Print help
"#;

fn parse(mut parser: lexopt::Parser) -> Result<CliOpt, lexopt::Error> {
  use lexopt::prelude::*;

  // Arguments
  let mut file: Vec<PathBuf> = vec![];

  while let Some(arg) = parser.next()? {
    match arg {
      Short('h') | Long("help") => {
        println!("{HELP}");
        std::process::exit(0);
      }
      Short('V') | Long("version") => {
        let pkg_version = env!("CARGO_PKG_VERSION");
        println!("rsvim {} (v8 {})", pkg_version, v8_version());
        std::process::exit(0);
      }
      Value(filename) => {
        file.push(Path::new(&filename).to_path_buf());
      }
      _ => return Err(arg.unexpected()),
    }
  }

  Ok(CliOpt { file })
}

impl CliOpt {
  pub fn from_env() -> Result<Self, lexopt::Error> {
    parse(lexopt::Parser::from_env())
  }

  pub fn from_args(
    args: Vec<std::ffi::OsString>,
  ) -> Result<Self, lexopt::Error> {
    parse(lexopt::Parser::from_args(args))
  }

  /// Input files.
  pub fn file(&self) -> &Vec<PathBuf> {
    &self.file
  }
}
