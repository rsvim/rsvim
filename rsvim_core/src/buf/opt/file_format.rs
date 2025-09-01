//! The "file-format" option for Vim buffer.

#[derive(
  Debug,
  Copy,
  Clone,
  PartialEq,
  Eq,
  strum_macros::Display,
  strum_macros::EnumString,
)]
pub enum FileFormatOption {
  #[strum(serialize = "dos")]
  /// CRLF
  Dos,

  #[strum(serialize = "unix")]
  /// LF
  Unix,

  #[strum(serialize = "mac")]
  /// CR
  /// NOTE: This is a legacy and actually not used today.
  Mac,
}

impl From<EndOfLineOption> for FileFormatOption {
  fn from(value: EndOfLineOption) -> Self {
    match value {
      EndOfLineOption::Crlf => FileFormatOption::Dos,
      EndOfLineOption::Lf => FileFormatOption::Unix,
      EndOfLineOption::Cr => FileFormatOption::Mac,
    }
  }
}

#[derive(
  Debug,
  Copy,
  Clone,
  PartialEq,
  Eq,
  strum_macros::Display,
  strum_macros::EnumString,
)]
pub enum EndOfLineOption {
  #[strum(serialize = "\r\n")]
  /// Windows
  Crlf,

  #[strum(serialize = "\n")]
  /// Unix
  Lf,

  #[strum(serialize = "\r")]
  /// Mac
  Cr,
}

// impl Display for EndOfLineOption {
//   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//     use crate::defaults::ascii::end_of_line as eol;
//
//     match self {
//       EndOfLineOption::CRLF => write!(f, "{}", eol::CRLF),
//       EndOfLineOption::LF => write!(f, "{}", eol::LF),
//       EndOfLineOption::CR => write!(f, "{}", eol::CR),
//     }
//   }
// }
//
// impl TryFrom<&str> for EndOfLineOption {
//   type Error = String;
//
//   fn try_from(value: &str) -> Result<Self, Self::Error> {
//     let lower_value = value.to_lowercase();
//     match lower_value.as_str() {
//       "CRLF" => Ok(EndOfLineOption::CRLF),
//       "LF" => Ok(EndOfLineOption::LF),
//       "CR" => Ok(EndOfLineOption::CR),
//       _ => Err("Unknown EndOfLine value".to_string()),
//     }
//   }
// }

impl From<FileFormatOption> for EndOfLineOption {
  fn from(value: FileFormatOption) -> Self {
    match value {
      FileFormatOption::Dos => EndOfLineOption::Crlf,
      FileFormatOption::Unix => EndOfLineOption::Lf,
      FileFormatOption::Mac => EndOfLineOption::Cr,
    }
  }
}
