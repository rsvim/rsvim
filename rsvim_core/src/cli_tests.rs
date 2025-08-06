use super::cli::*;

use std::path::Path;

#[test]
fn cli_opt1() {
  let input = vec![
    vec![],
    vec!["README.md"],
    vec!["README.md", "LICENSE"],
    vec!["README.md", "LICENSE", "--headless"],
    vec!["--headless", "README.md"],
  ]
  .iter()
  .map(|strings| {
    strings
      .iter()
      .map(|s| std::ffi::OsString::from(s.to_string()))
      .collect::<Vec<_>>()
  })
  .collect::<Vec<_>>();

  let to_pb = |paths: Vec<&str>| {
    paths
      .iter()
      .map(|p| Path::new(p).to_path_buf())
      .collect::<Vec<_>>()
  };

  let expects = vec![
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
