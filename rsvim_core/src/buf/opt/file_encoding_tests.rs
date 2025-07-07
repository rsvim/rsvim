use super::file_encoding::*;

#[test]
fn display1() {
  let actual1 = format!("{}", FileEncodingOption::Utf8);
  assert_eq!(actual1, "utf-8");
}
