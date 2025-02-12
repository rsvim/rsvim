//! Command line.

use clap::Parser;

// #[clap(
//   value_name = "CMD",
//   long = "cmd",
//   help = "Execute <CMD> before loading any config"
// )]
// cmd_before: Option<Vec<String>>,
//
// #[clap(
//   value_name = "CMD",
//   short = 'c',
//   help = "Execute <CMD> after loading config and first file"
// )]
// cmd_after: Option<Vec<String>>,
//
// #[arg(short = 'd', long, help = "Run in diff mode")]
// diff: bool,
//
// #[arg(long, help = "Run in headless mode, without a user interface")]
// headless: bool,
//
// #[arg(long, help = "Run in verbose mode")]
// verbose: bool,
//
// #[arg(long, help = "Run in debug mode")]
// debug: bool,

const ABOUT: &str = "The VIM editor reinvented in Rust+TypeScript.";
const AFTER_ABOUT: &str = "Copyright © 2025 RSVIM, VIM LICENSE.\nPlease checkout https://rsvim.github.io for more documentation.";

#[derive(Parser, Debug, Clone, Default)]
#[command(
  disable_version_flag = true,
  about = ABOUT,
  long_about = ABOUT,
  after_help = AFTER_ABOUT,
  after_long_help = AFTER_ABOUT
)]
/// Command line options.
pub struct CliOpt {
  #[arg(help = "Edit file(s)")]
  file: Vec<String>,

  #[arg(short = 'V', long = "version", help = "Print version")]
  version: bool,
}

impl CliOpt {
  /// Input files.
  pub fn file(&self) -> &Vec<String> {
    &self.file
  }

  /// Version.
  pub fn version(&self) -> bool {
    self.version
  }

  // /// Commands should be execute before loading any config.
  // pub fn cmd_before(&self) -> &Option<Vec<String>> {
  //   &self.cmd_before
  // }
  //
  // /// Commands should be execute after loading any config and first line.
  // pub fn cmd_after(&self) -> &Option<Vec<String>> {
  //   &self.cmd_after
  // }
  //
  // /// Run in diff mode.
  // pub fn diff(&self) -> bool {
  //   self.diff
  // }
  //
  // /// Run in headless mode, without TUI.
  // pub fn headless(&self) -> bool {
  //   self.headless
  // }
  //
  // /// Run in verbose mode.
  // pub fn verbose(&self) -> bool {
  //   self.verbose
  // }
  //
  // /// Run in debug mode.
  // pub fn debug(&self) -> bool {
  //   self.debug
  // }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn cli_opt1() {
    let input = [
      vec!["rsvim".to_string()],
      vec!["rsvim".to_string(), "--version".to_string()],
      vec!["rsvim".to_string(), "README.md".to_string()],
    ];

    let expect = [
      CliOpt {
        file: vec![],
        version: false,
      },
      CliOpt {
        file: vec![],
        version: true,
      },
      CliOpt {
        file: vec!["README.md".to_string()],
        version: false,
      },
    ];

    assert_eq!(input.len(), expect.len());
    let n = input.len();
    for i in 0..n {
      let actual = CliOpt::parse_from(&input[i]);
      assert_eq!(actual.file, expect[i].file);
      assert_eq!(actual.version(), expect[i].version());
    }
  }
}
