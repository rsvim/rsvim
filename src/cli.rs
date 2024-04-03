use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
pub struct Cli {
  #[arg(help = "Edit file(s)")]
  file: Vec<String>,

  #[arg(short, long)]
  debug: bool,

  #[clap(short = 'c', long)]
  cmd: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_cli() {
    let actual = Cli::parse_from(vec![] as Vec<String>);
    println!("actual-1: {:?}", actual);
    assert_eq!(actual.debug, false);
    assert_eq!(actual.cmd, None);
    assert_eq!(actual.file, vec![] as Vec<String>);
    let actual = Cli::parse_from(vec!["--debug"]);
    println!("actual-2: {:?}", actual);
    assert_eq!(actual.debug, true);
    assert_eq!(actual.cmd, None);
    assert_eq!(actual.file, vec![] as Vec<String>);
    let actual = Cli::parse_from(vec!["--version"]);
    println!("actual-3: {:?}", actual);
    assert_eq!(actual.debug, false);
    assert_eq!(actual.cmd, None);
    assert_eq!(actual.file, vec![] as Vec<String>);
    let actual = Cli::parse_from(vec!["README.md"]);
    println!("actual-4: {:?}", actual);
    assert_eq!(actual.debug, true);
    assert_eq!(actual.cmd, None);
    assert_eq!(actual.file, vec!["README.md".to_string()]);
    let actual = Cli::parse_from(vec!["--debug", "README.md"]);
    println!("actual-5: {:?}", actual);
    assert_eq!(actual.debug, true);
    assert_eq!(actual.cmd, None);
    assert_eq!(actual.file, vec!["README.md".to_string()]);
    let actual = Cli::parse_from(vec!["README.md", "LICENSE"]);
    println!("actual-6: {:?}", actual);
    assert_eq!(actual.debug, false);
    assert_eq!(actual.cmd, None);
    assert_eq!(
      actual.file,
      vec!["README.md".to_string(), "LICENSE".to_string()]
    );
    let actual = Cli::parse_from(vec!["README.md", "LICENSE", "--debug"]);
    println!("actual-7: {:?}", actual);
    assert_eq!(actual.debug, true);
    assert_eq!(actual.cmd, None);
    assert_eq!(
      actual.file,
      vec!["README.md".to_string(), "LICENSE".to_string()]
    );
    let actual = Cli::parse_from(vec!["README.md", "LICENSE", "--cmd", "echo 1"]);
    println!("actual-8: {:?}", actual);
    assert_eq!(actual.debug, false);
    assert_eq!(actual.cmd, Some(vec!["echo 1".to_string()]));
    assert_eq!(
      actual.file,
      vec!["README.md".to_string(), "LICENSE".to_string()]
    );
    let actual = Cli::parse_from(vec![
      "README.md",
      "LICENSE",
      "--cmd",
      "echo 1",
      "--cmd",
      "quit",
    ]);
    println!("actual-9: {:?}", actual);
    assert_eq!(actual.debug, false);
    assert_eq!(
      actual.cmd,
      Some(vec!["echo 1".to_string(), "quit".to_string()])
    );
    assert_eq!(
      actual.file,
      vec!["README.md".to_string(), "LICENSE".to_string()]
    );
  }
}
