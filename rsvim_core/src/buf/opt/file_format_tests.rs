use super::file_format::*;

#[test]
fn display1() {
  let actual1 = format!("{}", FileFormatOption::Dos);
  assert_eq!(actual1, "dos");
}
