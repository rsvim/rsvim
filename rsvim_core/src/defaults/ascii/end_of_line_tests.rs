use std::str::FromStr;

use super::end_of_line::*;

#[test]
fn display() {
  assert_eq!("\r\n", EndOfLine::Crlf.to_string());
  assert_eq!("\r\n", format!("{}", EndOfLine::Crlf));

  assert_eq!("\n", EndOfLine::Lf.to_string());
  assert_eq!("\n", format!("{}", EndOfLine::Lf));

  assert_eq!("\r", EndOfLine::Cr.to_string());
  assert_eq!("\r", format!("{}", EndOfLine::Cr));
}

#[test]
fn from_str() {
  assert_eq!(EndOfLine::from_str("\r\n").unwrap(), EndOfLine::Crlf);
  assert_eq!(EndOfLine::from_str("\n").unwrap(), EndOfLine::Lf);
  assert_eq!(EndOfLine::from_str("\r").unwrap(), EndOfLine::Cr);
  assert!(EndOfLine::from_str("a").is_err());
}
