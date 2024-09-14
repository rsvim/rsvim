//! Command line.

use clap::Parser;

// #[clap(
//   value_name = "CMD",
//   long = "cmd",
//   help = "Execute <CMD> before loading any config"
// )]
// cmd_before: Option<Vec<String>>,

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about=None)]
/// Command line options.
pub struct CliOpt {
  #[arg(help = "Edit file(s)")]
  file: Vec<String>,

  #[clap(
    value_name = "CMD",
    short = 'c',
    help = "Execute <CMD> after loading config and first file"
  )]
  cmd_after: Option<Vec<String>>,

  #[arg(short = 'd', long, help = "Run in diff mode")]
  diff: bool,

  #[arg(long, help = "Run in headless mode, without a user interface")]
  headless: bool,

  #[arg(long, help = "Run in verbose mode")]
  verbose: bool,

  #[arg(long, help = "Run in debug mode")]
  debug: bool,
}

impl CliOpt {
  /// Input files.
  pub fn file(&self) -> &Vec<String> {
    &self.file
  }

  // /// Commands should be execute before loading any config.
  // pub fn cmd_before(&self) -> &Option<Vec<String>> {
  //   &self.cmd_before
  // }

  /// Commands should be execute after loading any config and first line.
  pub fn cmd_after(&self) -> &Option<Vec<String>> {
    &self.cmd_after
  }

  /// Run in diff mode.
  pub fn diff(&self) -> bool {
    self.diff
  }

  /// Run in headless mode, without TUI.
  pub fn headless(&self) -> bool {
    self.headless
  }

  /// Run in verbose mode.
  pub fn verbose(&self) -> bool {
    self.verbose
  }

  /// Run in debug mode.
  pub fn debug(&self) -> bool {
    self.debug
  }
}

#[cfg(test)]
mod tests {
  // use super::*;

  #[test]
  fn cli_opt1() {
    // let input = vec![
    //   vec!["rsvim".to_string()],
    //   vec![
    //     "rsvim".to_string(),
    //     "--version".to_string(),
    //     "--headless".to_string(),
    //     "--debug".to_string(),
    //     "-d".to_string(),
    //   ],
    //   vec!["rsvim".to_string(), "README.md".to_string()],
    //   vec![
    //     "rsvim".to_string(),
    //     "README.md".to_string(),
    //     "LICENSE".to_string(),
    //     "--headless".to_string(),
    //     "-d".to_string(),
    //   ],
    //   vec![
    //     "rsvim".to_string(),
    //     "README.md".to_string(),
    //     "LICENSE".to_string(),
    //     "--cmd".to_string(),
    //     "echo 1".to_string(),
    //     "-c".to_string(),
    //     "quit".to_string(),
    //   ],
    // ] as Vec<Vec<String>>;
    // let expect = vec![
    //   CliOpt {
    //     file: vec![],
    //     cmd_before: None,
    //     cmd_after: None,
    //     diff: false,
    //     headless: false,
    //     verbose: false,
    //     debug: false,
    //   },
    //   CliOpt {
    //     file: vec![],
    //     cmd_before: None,
    //     cmd_after: None,
    //     diff: true,
    //     headless: true,
    //     verbose: false,
    //     debug: true,
    //   },
    //   CliOpt {
    //     file: vec!["README.md".to_string()],
    //     cmd_before: None,
    //     cmd_after: None,
    //     diff: false,
    //     headless: false,
    //     verbose: false,
    //     debug: false,
    //   },
    //   CliOpt {
    //     file: vec!["README.md".to_string(), "LICENSE".to_string()],
    //     cmd_before: None,
    //     cmd_after: None,
    //     diff: true,
    //     headless: true,
    //     verbose: false,
    //     debug: false,
    //   },
    //   CliOpt {
    //     file: vec!["README.md".to_string(), "LICENSE".to_string()],
    //     cmd_before: Some(vec!["echo 1".to_string()]),
    //     cmd_after: Some(vec!["quit".to_string()]),
    //     diff: false,
    //     headless: false,
    //     verbose: false,
    //     debug: false,
    //   },
    // ];
    //
    // assert_eq!(input.len(), expect.len());
    // let n = input.len();
    // for i in 0..n {
    //   let actual = CliOpt::parse_from(&input[i]);
    //   // println!("actual-{i}: {:?}", actual);
    //   // println!("expect-{i}: {:?}", expect[i]);
    //   assert_eq!(actual.file, expect[i].file);
    //   assert_eq!(actual.cmd_before, expect[i].cmd_before);
    //   assert_eq!(actual.cmd_after, expect[i].cmd_after);
    //   assert_eq!(actual.diff, expect[i].diff);
    //   assert_eq!(actual.headless, expect[i].headless);
    //   assert_eq!(actual.debug, expect[i].debug);
    // }
  }
}
