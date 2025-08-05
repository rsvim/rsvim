//! Command line options.

use crate::js::v8_version;

use std::{
  alloc::handle_alloc_error,
  path::{Path, PathBuf},
};

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
  fn handle_error(result: Result<Self, lexopt::Error>) -> Self {
    match result {
      Ok(res) => res,
      Err(e) => {
        println!("error: {e}");
        println!("");
        println!("For more information, try '--help'");
        std::process::exit(0);
      }
    }
  }

  pub fn from_env() -> Self {
    let result = parse(lexopt::Parser::from_env());
    Self::handle_error(result)
  }

  pub fn from_args(args: Vec<std::ffi::OsString>) -> Self {
    let result = parse(lexopt::Parser::from_args(args));
    Self::handle_error(result)
  }

  /// Input files.
  pub fn file(&self) -> &Vec<PathBuf> {
    &self.file
  }
}
