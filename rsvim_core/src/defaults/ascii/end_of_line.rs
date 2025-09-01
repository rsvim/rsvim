//! End of line.

use std::str::FromStr;
// use std::string::ToString;
// use strum_macros::{Display, EnumString};

#[derive(
  Debug, PartialEq, Eq, strum_macros::Display, strum_macros::EnumString,
)]
pub enum EndOfLine {
  #[strum(serialize = "\r\n")]
  #[strum(to_string = "\r\n")]
  Crlf,

  #[strum(serialize = "\n")]
  #[strum(to_string = "\n")]
  Lf,

  #[strum(serialize = "\r")]
  #[strum(to_string = "\r")]
  Cr,
}

// pub const CRLF: &str = "\r\n";
// pub const LF: &str = "\n";
// pub const CR: &str = "\r";
