use super::file_format::*;

use std::str::FromStr;

#[test]
fn display1() {
  let actual1 = format!("{}", FileFormatOption::Dos);
  assert_eq!(actual1, "dos");
}

#[test]
fn display2() {
  assert_eq!("\r\n", EndOfLineOption::Crlf.to_string());
  assert_eq!("\r\n", format!("{}", EndOfLineOption::Crlf));

  assert_eq!("\n", EndOfLineOption::Lf.to_string());
  assert_eq!("\n", format!("{}", EndOfLineOption::Lf));

  assert_eq!("\r", EndOfLineOption::CR.to_string());
  assert_eq!("\r", format!("{}", EndOfLineOption::CR));
}

#[test]
fn from_str() {
  assert_eq!(
    EndOfLineOption::from_str("\r\n").unwrap(),
    EndOfLineOption::Crlf
  );
  assert_eq!(
    EndOfLineOption::from_str("\n").unwrap(),
    EndOfLineOption::Lf
  );
  assert_eq!(
    EndOfLineOption::from_str("\r").unwrap(),
    EndOfLineOption::CR
  );
  assert!(EndOfLineOption::from_str("a").is_err());
}
