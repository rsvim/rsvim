use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
pub struct Cli {
  #[arg(help = "Edit file(s)")]
  file: Vec<String>,

  #[clap(
    value_name = "CMD",
    long = "cmd",
    help = "Execute <CMD> before loading any config"
  )]
  cmd_before: Option<Vec<String>>,

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

impl Cli {
  pub fn file(&self) -> Vec<&str> {
    self.file.iter().map(|f| &**f).collect()
  }

  pub fn cmd_before(&self) -> Option<Vec<&str>> {
    self
      .cmd_before
      .as_ref()
      .map(|cb| cb.iter().map(|c| &**c).collect())
  }

  pub fn cmd_after(&self) -> Option<Vec<&str>> {
    self
      .cmd_after
      .as_ref()
      .map(|ca| ca.iter().map(|c| &**c).collect())
  }

  pub fn diff(&self) -> bool {
    self.diff
  }

  pub fn headless(&self) -> bool {
    self.headless
  }

  pub fn verbose(&self) -> bool {
    self.verbose
  }

  pub fn debug(&self) -> bool {
    self.debug
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_cli() {
    let input = vec![
      vec![],
      vec![
        "--version".to_string(),
        "--headless".to_string(),
        "--debug".to_string(),
        "-d".to_string(),
      ],
      vec!["README.md".to_string()],
      vec![
        "README.md".to_string(),
        "LICENSE".to_string(),
        "--headless".to_string(),
        "-d".to_string(),
      ],
      vec![
        "README.md".to_string(),
        "LICENSE".to_string(),
        "--cmd".to_string(),
        "echo 1".to_string(),
        "-c".to_string(),
        "quit".to_string(),
      ],
    ] as Vec<Vec<String>>;
    let expect = vec![
      Cli {
        file: vec![],
        cmd_before: None,
        cmd_after: None,
        diff: false,
        headless: false,
        verbose: false,
        debug: false,
      },
      Cli {
        file: vec![],
        cmd_before: None,
        cmd_after: None,
        diff: true,
        headless: true,
        verbose: false,
        debug: true,
      },
      Cli {
        file: vec!["README.md".to_string()],
        cmd_before: None,
        cmd_after: None,
        diff: false,
        headless: false,
        verbose: false,
        debug: false,
      },
      Cli {
        file: vec!["README.md".to_string(), "LICENSE".to_string()],
        cmd_before: None,
        cmd_after: None,
        diff: true,
        headless: true,
        verbose: false,
        debug: false,
      },
      Cli {
        file: vec!["README.md".to_string(), "LICENSE".to_string()],
        cmd_before: Some(vec!["echo 1".to_string()]),
        cmd_after: Some(vec!["quit".to_string()]),
        diff: false,
        headless: false,
        verbose: false,
        debug: false,
      },
    ];

    assert_eq!(input.len(), expect.len());
    let n = input.len();
    for i in 0..n {
      let actual = Cli::parse_from(&input[i]);
      println!("actual-{i}: {:?}", actual);
      println!("expect-{i}: {:?}", expect[i]);
      assert_eq!(actual.file, expect[i].file);
      assert_eq!(actual.cmd_before, expect[i].cmd_before);
      assert_eq!(actual.cmd_after, expect[i].cmd_after);
      assert_eq!(actual.diff, expect[i].diff);
      assert_eq!(actual.headless, expect[i].headless);
      assert_eq!(actual.debug, expect[i].debug);
    }
  }
}
