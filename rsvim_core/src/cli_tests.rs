use super::cli::*;

use std::path::Path;

#[test]
fn cli_opt1() {
  let to_os_str = |osstrs: Vec<&str>| {
    osstrs
      .iter()
      .map(|s| std::ffi::OsString::from(s.to_string()))
      .collect::<Vec<_>>()
  };

  let input = [
    to_os_str(vec![]),
    to_os_str(vec!["README.md"]),
    to_os_str(vec!["README.md", "LICENSE"]),
    to_os_str(vec!["README.md", "LICENSE", "--headless"]),
    to_os_str(vec!["--headless", "README.md"]),
  ];

  let to_pb = |paths: Vec<&str>| {
    paths
      .iter()
      .map(|p| Path::new(p).to_path_buf())
      .collect::<Vec<_>>()
  };

  let expects = [
    CliOptions::new(vec![], false),
    CliOptions::new(to_pb(vec!["README.md"]), false),
    CliOptions::new(to_pb(vec!["README.md", "LICENSE"]), false),
    CliOptions::new(to_pb(vec!["README.md", "LICENSE"]), true),
    CliOptions::new(to_pb(vec!["README.md"]), true),
  ];

  assert_eq!(input.len(), expects.len());
  let n = input.len();
  for i in 0..n {
    let actual = CliOptions::from_args(&input[i]);
    let expect = &expects[i];
    assert_eq!(actual.headless(), expect.headless());
    assert_eq!(actual.file().len(), expect.file().len());
    for (j, act) in actual.file().iter().enumerate() {
      assert_eq!(act, &expect.file()[j]);
    }
  }
}
