use super::cli::*;

use std::path::Path;

#[test]
fn cli_opt1() {
  let input = [
    vec![],
    vec!["README.md"],
    vec!["README.md", "LICENSE"],
    vec!["README.md", "LICENSE", "--help", "--version"],
    vec!["README.md", "LICENSE", "-h", "-V"],
    vec!["README.md", "LICENSE", "--headless"],
  ];

  let to_pb = |paths: Vec<&str>| {
    paths
      .iter()
      .map(|p| Path::new(p).to_path_buf())
      .collect::<Vec<_>>()
  };

  let expects = [
    CliOptions::new(CliSpecialOptions::empty(), to_pb(vec![]), false),
    CliOptions::new(
      CliSpecialOptions::empty(),
      to_pb(vec!["README.md"]),
      false,
    ),
    CliOptions::new(
      CliSpecialOptions::empty(),
      to_pb(vec!["README.md", "LICENSE"]),
      false,
    ),
    CliOptions::new(
      CliSpecialOptions::new(true, false, true),
      to_pb(vec!["README.md", "LICENSE"]),
      false,
    ),
    CliOptions::new(
      CliSpecialOptions::new(true, true, false),
      to_pb(vec!["README.md", "LICENSE"]),
      false,
    ),
    CliOptions::new(
      CliSpecialOptions::empty(),
      to_pb(vec!["README.md", "LICENSE"]),
      true,
    ),
  ];

  assert_eq!(input.len(), expects.len());
  let n = input.len();
  for i in 0..n {
    let actual = CliOptions::from_args(&input[i]).unwrap();
    let expect = &expects[i];
    assert_eq!(actual.headless(), expect.headless());
    assert_eq!(actual.file().len(), expect.file().len());
    for (j, act) in actual.file().iter().enumerate() {
      assert_eq!(act, &expect.file()[j]);
    }
    assert_eq!(actual.special_opts(), expect.special_opts());
  }
}

#[test]
fn cli_opt2() {
  let input = [vec!["--ex"], vec!["--v"]];

  for i in input {
    let actual = CliOptions::from_args(&i);
    assert!(actual.is_err());
  }
}
