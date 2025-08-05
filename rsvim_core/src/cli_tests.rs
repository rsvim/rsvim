use super::cli::*;

use std::path::Path;

#[test]
fn cli_opt1() {
  let input: [Vec<std::ffi::OsString>; 3] = [
    vec!["rsvim".to_string().into()],
    vec!["rsvim".to_string().into(), "--version".to_string().into()],
    vec!["rsvim".to_string().into(), "README.md".to_string().into()],
  ];

  let expect = [vec![], vec![], vec![Path::new("README.md").to_path_buf()]];

  assert_eq!(input.len(), expect.len());
  let n = input.len();
  for i in 0..n {
    let actual = CliOptions::from_args(&input[i]);
    assert_eq!(actual.file().len(), expect[i].len());
    for (j, act) in actual.file().iter().enumerate() {
      assert_eq!(act, &expect[i][j]);
    }
  }
}
