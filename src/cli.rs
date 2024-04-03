use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about=None)]
pub struct Cli {
  #[arg(short, long, action=clap::ArgAction::Count)]
  debug: bool,

  #[clap(short='c', long, num_args=0..)]
  cmd: Option<Vec<String>>,

  #[clap(short, long, value_delimiter=' ', num_args=0..)]
  file: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_cli() {
    let actual = Cli::parse_from(vec![] as Vec<String>);
    assert_eq!(actual.debug, false);
    assert_eq!(actual.cmd, None);
    assert_eq!(actual.file, None);
    let actual = Cli::parse_from(vec!["--debug"]);
    assert_eq!(actual.debug, true);
    assert_eq!(actual.cmd, None);
    assert_eq!(actual.file, None);
    let actual = Cli::parse_from(vec!["--version"]);
    assert_eq!(actual.debug, false);
    assert_eq!(actual.cmd, None);
    assert_eq!(actual.file, None);
    let actual = Cli::parse_from(vec!["README.md"]);
    assert_eq!(actual.debug, true);
    assert_eq!(actual.cmd, None);
    assert_eq!(actual.file, Some(vec!["README.md".to_string()]));
    let actual = Cli::parse_from(vec!["--debug", "README.md"]);
    assert_eq!(actual.debug, true);
    assert_eq!(actual.cmd, None);
    assert_eq!(actual.file, Some(vec!["README.md".to_string()]));
    let actual = Cli::parse_from(vec!["README.md", "LICENSE"]);
    assert_eq!(actual.debug, false);
    assert_eq!(actual.cmd, None);
    assert_eq!(
      actual.file,
      Some(vec!["README.md".to_string(), "LICENSE".to_string()])
    );
    let actual = Cli::parse_from(vec!["README.md", "LICENSE", "--debug"]);
    assert_eq!(actual.debug, true);
    assert_eq!(actual.cmd, None);
    assert_eq!(
      actual.file,
      Some(vec!["README.md".to_string(), "LICENSE".to_string()])
    );
    let actual = Cli::parse_from(vec!["README.md", "LICENSE", "--cmd", "echo 1"]);
    assert_eq!(actual.debug, false);
    assert_eq!(actual.cmd, Some(vec!["echo 1".to_string()]));
    assert_eq!(
      actual.file,
      Some(vec!["README.md".to_string(), "LICENSE".to_string()])
    );
    let actual = Cli::parse_from(vec![
      "README.md",
      "LICENSE",
      "--cmd",
      "echo 1",
      "--cmd",
      "quit",
    ]);
    assert_eq!(actual.debug, false);
    assert_eq!(
      actual.cmd,
      Some(vec!["echo 1".to_string(), "quit".to_string()])
    );
    assert_eq!(
      actual.file,
      Some(vec!["README.md".to_string(), "LICENSE".to_string()])
    );
  }
}
