use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
pub struct Cli {
  #[arg(help = "Edit file(s)")]
  file: Vec<String>,

  #[arg(long, help = "Run in debug mode")]
  debug: bool,

  #[arg(long, help = "Run in headless mode, without a user interface")]
  headless: bool,

  #[clap(
    value_name = "CMD",
    long = "cmd",
    help = "Execute <CMD> before loading any config"
  )]
  cmd_before_config: Option<Vec<String>>,

  #[clap(
    value_name = "CMD",
    short = 'c',
    help = "Execute <CMD> after loading config and first file"
  )]
  cmd_after_config: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_cli() {
    let actual = Cli::parse_from(vec![] as Vec<String>);
    println!("actual-1: {:?}", actual);
    assert_eq!(actual.debug, false);
    assert_eq!(actual.headless, false);
    assert_eq!(actual.cmd_before_config, None);
    assert_eq!(actual.cmd_after_config, None);
    assert_eq!(actual.file, vec![] as Vec<String>);
    let actual = Cli::parse_from(vec!["--version", "--headless"]);
    println!("actual-3: {:?}", actual);
    assert_eq!(actual.debug, false);
    assert_eq!(actual.cmd_before_config, None);
    assert_eq!(actual.cmd_after_config, None);
    assert_eq!(actual.file, vec![] as Vec<String>);
    let actual = Cli::parse_from(vec!["README.md"]);
    println!("actual-4: {:?}", actual);
    assert_eq!(actual.debug, false);
    assert_eq!(actual.cmd_before_config, None);
    assert_eq!(actual.cmd_after_config, None);
    assert_eq!(actual.file, vec!["README.md".to_string()]);
    let actual = Cli::parse_from(vec!["--debug", "README.md"]);
    println!("actual-5: {:?}", actual);
    assert_eq!(actual.debug, true);
    assert_eq!(actual.cmd_before_config, None);
    assert_eq!(actual.cmd_after_config, None);
    assert_eq!(actual.file, vec!["README.md".to_string()]);
    let actual = Cli::parse_from(vec!["README.md", "LICENSE"]);
    println!("actual-6: {:?}", actual);
    assert_eq!(actual.debug, false);
    assert_eq!(actual.cmd_before_config, None);
    assert_eq!(actual.cmd_after_config, None);
    assert_eq!(
      actual.file,
      vec!["README.md".to_string(), "LICENSE".to_string()]
    );
    let actual = Cli::parse_from(vec!["README.md", "LICENSE", "--debug"]);
    println!("actual-7: {:?}", actual);
    assert_eq!(actual.debug, true);
    assert_eq!(actual.cmd_before_config, None);
    assert_eq!(actual.cmd_after_config, None);
    assert_eq!(
      actual.file,
      vec!["README.md".to_string(), "LICENSE".to_string()]
    );
    let actual = Cli::parse_from(vec!["README.md", "LICENSE", "--cmd", "echo 1"]);
    println!("actual-8: {:?}", actual);
    assert_eq!(actual.debug, false);
    assert_eq!(actual.cmd_before_config, Some(vec!["echo 1".to_string()]));
    assert_eq!(actual.cmd_after_config, None);
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
      actual.cmd_before_config,
      Some(vec!["echo 1".to_string(), "quit".to_string()])
    );
    assert_eq!(actual.cmd_after_config, None);
    assert_eq!(
      actual.file,
      vec!["README.md".to_string(), "LICENSE".to_string()]
    );
    let actual = Cli::parse_from(vec![
      "README.md",
      "LICENSE",
      "-c",
      "echo 1",
      "--cmd",
      "quit",
    ]);
    println!("actual-10: {:?}", actual);
    assert_eq!(actual.debug, false);
    assert_eq!(actual.cmd_before_config, Some(vec!["quit".to_string()]));
    assert_eq!(actual.cmd_after_config, Some(vec!["echo 1".to_string()]));
    assert_eq!(
      actual.file,
      vec!["README.md".to_string(), "LICENSE".to_string()]
    );
    let actual = Cli::parse_from(vec!["--headless", "LICENSE"]);
    println!("actual-11: {:?}", actual);
    assert_eq!(actual.headless, true);
    assert_eq!(actual.file, vec!["LICENSE".to_string()]);
  }
}
