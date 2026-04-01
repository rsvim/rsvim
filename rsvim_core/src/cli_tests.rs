use super::cli::*;
use clap::Parser;
use std::path::Path;

#[test]
fn cli_opt1() {
  let input = [
    vec![],
    vec!["README.md"],
    vec!["README.md", "LICENSE"],
    vec!["README.md", "LICENSE", "--help", "--version"],
    vec!["README.md", "LICENSE", "-h", "-V"],
    vec!["README.md", "LICENSE"],
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
    CliOptions::new(true, to_pathbuf(vec!["README.md", "LICENSE"])),
    CliOptions::new(false, to_pathbuf(vec!["README.md", "LICENSE"])),
  ];

  assert_eq!(input.len(), expects.len());
  let n = input.len();
  for i in 0..n {
    let actual = CliOptions::try_parse_from(&input[i]).unwrap();
    let expect = &expects[i];
    assert_eq!(actual.file().len(), expect.file().len());
    for (j, act) in actual.file().iter().enumerate() {
      assert_eq!(act, &expect.file()[j]);
    }
  }
}

#[test]
fn cli_opt2() {
  let input = [vec!["--ex"], vec!["--v"]];

  for i in input {
    let actual = CliOptions::try_parse_from(&i).is_err();
    assert!(actual);
  }
}
