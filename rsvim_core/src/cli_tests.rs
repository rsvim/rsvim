use super::cli::*;

use std::path::Path;

#[test]
fn cli_opt1() {
  let to_osstr = |osstrs: Vec<&str>| {
    osstrs
      .iter()
      .map(|s| std::ffi::OsString::from(s.to_string()))
      .collect::<Vec<_>>()
  };

  let input = [
    to_osstr(vec![]),
    to_osstr(vec!["README.md"]),
    to_osstr(vec!["README.md", "LICENSE"]),
    to_osstr(vec!["README.md", "LICENSE", "--help", "--version"]),
    to_osstr(vec!["README.md", "LICENSE", "-h", "-V"]),
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
  let to_osstr = |osstrs: Vec<&str>| {
    osstrs
      .iter()
      .map(|s| std::ffi::OsString::from(s.to_string()))
      .collect::<Vec<_>>()
  };

  let input = [to_osstr(vec!["--ex"]), to_osstr(vec!["--v"])];

  for i in input {
    let actual = CliOptions::from_args(&i);
    assert!(actual.is_err());
  }
}
