//! End of line.

// use std::str::FromStr;
// use std::string::ToString;
// use strum_macros::{Display, EnumString};

#[derive(Debug, PartialEq, Eq, strum_macros::Display)]
pub enum EndOfLine {
  #[strum(to_string = "\r\n")]
  CRLF,
  #[strum(to_string = "\n")]
  LF,
  #[strum(to_string = "\r")]
  CR,
}

// pub const CRLF: &str = "\r\n";
// pub const LF: &str = "\n";
// pub const CR: &str = "\r";
