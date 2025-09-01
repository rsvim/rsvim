use super::file_format::*;

#[test]
fn display1() {
  let actual1 = format!("{}", FileFormatOption::Dos);
  assert_eq!(actual1, "dos");
}

#[test]
fn display2() {
  assert_eq!("\r\n", EndOfLineOption::CRLF.to_string());
  assert_eq!("\r\n", format!("{}", EndOfLineOption::CRLF));

  assert_eq!("\n", EndOfLineOption::LF.to_string());
  assert_eq!("\n", format!("{}", EndOfLineOption::LF));

  assert_eq!("\r", EndOfLineOption::CR.to_string());
  assert_eq!("\r", format!("{}", EndOfLineOption::CR));
}

#[test]
fn from_str() {
  assert_eq!(
    EndOfLineOption::from_str("\r\n").unwrap(),
    EndOfLineOption::CRLF
  );
  assert_eq!(
    EndOfLineOption::from_str("\n").unwrap(),
    EndOfLineOption::LF
  );
  assert_eq!(
    EndOfLineOption::from_str("\r").unwrap(),
    EndOfLineOption::CR
  );
  assert!(EndOfLineOption::from_str("a").is_err());
}
