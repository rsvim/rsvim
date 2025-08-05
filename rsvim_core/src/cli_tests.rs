use super::cli::*;

use std::path::Path;

#[test]
fn cli_opt1() {
  let input: Vec<Vec<std::ffi::OsString>> = vec![
    vec![],
    vec!["README.md".to_string().into()],
    vec!["README.md".to_string().into(), "LICENSE".to_string().into()],
  ];

  let expect = [vec![], vec![Path::new("README.md").to_path_buf()],
    vec![Path::new("README.md").to_path_buf(), Path::new("LICENSE").to_path_buf()]
  ];

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
