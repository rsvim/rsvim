//! End of line.

// use std::string::ToString;
// use strum_macros::{Display, EnumString};

#[derive(
  Debug, PartialEq, Eq, strum_macros::Display, strum_macros::EnumString,
)]
pub enum EndOfLine {
  #[strum(serialize = "\r\n")]
  Crlf,

  #[strum(serialize = "\n")]
  Lf,

  #[strum(serialize = "\r")]
  Cr,
}

// pub const CRLF: &str = "\r\n";
// pub const LF: &str = "\n";
// pub const CR: &str = "\r";
