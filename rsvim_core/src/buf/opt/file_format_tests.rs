use super::file_format::*;
use std::str::FromStr;

#[test]
fn display1() {
  assert_eq!("dos", FileFormatOption::Dos.to_string());
  assert_eq!("dos", format!("{}", FileFormatOption::Dos));

  assert_eq!("unix", FileFormatOption::Unix.to_string());
  assert_eq!("unix", format!("{}", FileFormatOption::Unix));

  assert_eq!("mac", FileFormatOption::Mac.to_string());
  assert_eq!("mac", format!("{}", FileFormatOption::Mac));
}

#[test]
fn from_str1() {
  assert_eq!(
    FileFormatOption::from_str("dos").unwrap(),
    FileFormatOption::Dos
  );
  assert_eq!(
    FileFormatOption::from_str("unix").unwrap(),
    FileFormatOption::Unix
  );
  assert_eq!(
    FileFormatOption::from_str("mac").unwrap(),
    FileFormatOption::Mac
  );
  assert!(FileFormatOption::from_str("a").is_err());
}

#[test]
fn display2() {
  assert_eq!("\r\n", EndOfLineOption::Crlf.to_string());
  assert_eq!("\r\n", format!("{}", EndOfLineOption::Crlf));

  assert_eq!("\n", EndOfLineOption::Lf.to_string());
  assert_eq!("\n", format!("{}", EndOfLineOption::Lf));

  assert_eq!("\r", EndOfLineOption::Cr.to_string());
  assert_eq!("\r", format!("{}", EndOfLineOption::Cr));
}

#[test]
fn from_str2() {
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
    EndOfLineOption::Cr
  );
  assert!(EndOfLineOption::from_str("a").is_err());
}
