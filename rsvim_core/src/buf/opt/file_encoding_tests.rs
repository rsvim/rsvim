use super::file_encoding::*;

use std::str::FromStr;

#[test]
fn display1() {
  assert_eq!("utf-8", FileEncodingOption::Utf8.to_string());
  assert_eq!("utf-8", format!("{}", FileEncodingOption::Utf8));
}

#[test]
fn from_str1() {
  assert_eq!(
    FileEncodingOption::from_str("utf-8").unwrap(),
    FileEncodingOption::Utf8
  );
  assert!(FileEncodingOption::from_str("a").is_err());
}
