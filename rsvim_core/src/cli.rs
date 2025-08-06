//! Command line options.

use crate::js::v8_version;

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
/// Command line options.
pub struct CliOptions {
  file: Vec<PathBuf>,
  ex_mode: bool,
}

const SHORT_HELP: &str = r#"Usage: {RSVIM_BIN_NAME} [OPTIONS] [FILE]...

Arguments:
  [FILE]...  Edit file(s)

Options:
      --ex (experimental)  Run in ex mode without TUI
  -h, --help               Print help (see more with '--help')
  -V, --version            Print version
"#;

const LONG_HELP: &str = r#"The VIM editor reinvented in Rust+TypeScript

rsvim is an open source terminal based text editor, strives to be highly
extensible by following the main features and philosophy of (neo)vim. It is
built from scratch with rust, tokio and v8 javascript engine.

Project home page: https://github.com/rsvim/rsvim
Project documentation page: https://rsvim.github.io/

Usage: {RSVIM_BIN_NAME} [OPTIONS] [FILE]...

Arguments:
  [FILE]...
          Edit file(s)

Options:
      --ex (experimental)
          Run in ex mode without TUI. In this mode, rsvim doesn't enter
          terminal raw mode, it uses STDIN to receive javascript script, and
          uses STDOUT, STDERR to print messages instead of rendering TUI. All
          internal data structures (such as buffers, windows, command-line,
          etc) and scripts/plugins will still be initialized
 
  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Bugs can be reported on GitHub: https://github.com/rsvim/rsvim/issues
"#;

const VERSION: &str =
  "{RSVIM_BIN_NAME} {RSVIM_PKG_VERSION} (v8 {RSVIM_V8_VERSION})";

fn parse(mut parser: lexopt::Parser) -> Result<CliOptions, lexopt::Error> {
  use lexopt::prelude::*;

  let exe_name = std::env::current_exe().unwrap();
  let bin_name = exe_name.as_path().file_stem().unwrap().to_str().unwrap();

  // Arguments
  let mut file: Vec<PathBuf> = vec![];
  let mut ex_mode: bool = false;

  while let Some(arg) = parser.next()? {
    match arg {
      Short('h') | Long("help") => {
        let help = match arg {
          Short(_) => SHORT_HELP.replace("{RSVIM_BIN_NAME}", bin_name),
          Long(_) => LONG_HELP.replace("{RSVIM_BIN_NAME}", bin_name),
          _ => unreachable!(),
        };
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
      Long("ex") => {
        ex_mode = true;
      }
      Value(filename) => {
        file.push(Path::new(&filename).to_path_buf());
      }
      _ => return Err(arg.unexpected()),
    }
  }

  Ok(CliOptions { file, ex_mode })
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

  /// Ex mode.
  pub fn ex_mode(&self) -> bool {
    self.ex_mode
  }

  #[cfg(test)]
  pub fn new(file: Vec<PathBuf>, ex_mode: bool) -> Self {
    Self { file, ex_mode }
  }
}
