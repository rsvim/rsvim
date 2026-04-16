use super::cli::*;
use crate::prelude::*;
use crate::tests::log::init as test_log_init;
use clap::Parser;

#[test]
fn cli_opt1() {
  test_log_init();

  let input = [
    vec!["rsvim"],
    vec!["rsvim", "README.md"],
    vec!["rsvim", "README.md", "LICENSE"],
    vec!["rsvim", "README.md", "LICENSE", "--version"],
    vec!["rsvim", "README.md", "-V"],
  ];

  let to_pathbuf = |paths: Vec<&str>| {
    paths
      .iter()
      .map(|p| Path::new(p).to_path_buf())
      .collect::<Vec<_>>()
  };

  let expects = [
    CliOptions::new(false, to_pathbuf(vec![])),
    CliOptions::new(false, to_pathbuf(vec!["README.md"])),
    CliOptions::new(false, to_pathbuf(vec!["README.md", "LICENSE"])),
    CliOptions::new(true, to_pathbuf(vec!["README.md", "LICENSE"])),
    CliOptions::new(true, to_pathbuf(vec!["README.md"])),
  ];

  assert_eq!(input.len(), expects.len());
  let n = input.len();
  for i in 0..n {
    let actual = CliOptions::parse_from(input[i].iter());
    let expect = &expects[i];
    info!(
      "{} input:{:?},actual:{:?},expect:{:?}",
      i, input[i], actual, expect
    );
    assert_eq!(actual.version(), expect.version());
    assert_eq!(actual.file().len(), expect.file().len());
    for (j, act) in actual.file().iter().enumerate() {
      assert_eq!(act, &expect.file()[j]);
    }
  }
}

#[test]
fn cli_opt2() {
  test_log_init();
  let input = [vec!["rsvim", "--ex"], vec!["rsvim", "--v"]];

  for i in input {
    let actual = CliOptions::try_parse_from(&i);
    info!("input:{:?},actual:{:?}", i, actual);
    assert!(actual.is_err());
  }
}
