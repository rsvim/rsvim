use super::cli::*;

use clap::Parser;

#[test]
fn cli_opt1() {
  let input = [
    vec!["rsvim".to_string()],
    vec!["rsvim".to_string(), "--version".to_string()],
    vec!["rsvim".to_string(), "README.md".to_string()],
  ];

  let expect = [
    CliOpt::new(false, vec![]),
    CliOpt::new(true, vec![]),
    CliOpt::new(false, vec!["README.md".to_string()]),
  ];

  assert_eq!(input.len(), expect.len());
  let n = input.len();
  for i in 0..n {
    let actual = CliOpt::parse_from(&input[i]);
    assert_eq!(actual.file(), expect[i].file());
    assert_eq!(actual.version(), expect[i].version());
  }
}
