//! Command line options.

use crate::js::v8_version;

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
/// Command line options.
pub struct CliOptions {
  file: Vec<PathBuf>,
}

const HELP: &str = r#"The VIM editor reinvented in Rust+TypeScript

Usage: {RSVIM_BIN_NAME} [FILE]...

Arguments:
  [FILE]...  Edit file(s)

Options:
  -V, --version  Print version
  -h, --help     Print help
"#;

const VERSION: &str =
  "{RSVIM_BIN_NAME} {RSVIM_PKG_VERSION} (v8 {RSVIM_V8_VERSION})";

fn parse(mut parser: lexopt::Parser) -> Result<CliOptions, lexopt::Error> {
  use lexopt::prelude::*;

  let exe_name = std::env::current_exe().unwrap();
  let bin_name = exe_name.as_path().file_stem().unwrap().to_str().unwrap();

  // Arguments
  let mut file: Vec<PathBuf> = vec![];

  while let Some(arg) = parser.next()? {
    match arg {
      Short('h') | Long("help") => {
        let help = HELP.replace("{RSVIM_BIN_NAME}", bin_name);
        println!("{help}");
        std::process::exit(0);
      }
      Short('V') | Long("version") => {
        let version = VERSION
          .replace("{RSVIM_BIN_NAME}", bin_name)
          .replace("{RSVIM_PKG_VERSION}", env!("CARGO_PKG_VERSION"))
          .replace("{RSVIM_V8_VERSION}", v8_version());
        println!("{version}");
        std::process::exit(0);
      }
      Value(filename) => {
        file.push(Path::new(&filename).to_path_buf());
      }
      _ => return Err(arg.unexpected()),
    }
  }

  Ok(CliOptions { file })
}

impl CliOptions {
  fn handle_error(result: Result<Self, lexopt::Error>) -> Self {
    match result {
      Ok(res) => res,
      Err(e) => {
        println!("error: {e}");
        println!();
        println!("For more information, try '--help'");
        std::process::exit(0);
      }
    }
  }

  pub fn from_env() -> Self {
    let result = parse(lexopt::Parser::from_env());
    Self::handle_error(result)
  }

  pub fn from_args(args: &Vec<std::ffi::OsString>) -> Self {
    let result = parse(lexopt::Parser::from_args(args));
    Self::handle_error(result)
  }

  /// Input files.
  pub fn file(&self) -> &Vec<PathBuf> {
    &self.file
  }
}
