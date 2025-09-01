//! The "file-encoding" option for Vim buffer.

#[derive(
  Debug,
  Copy,
  Clone,
  PartialEq,
  Eq,
  strum_macros::Display,
  strum_macros::EnumString,
)]
pub enum FileEncodingOption {
  #[strum(serialize = "utf-8")]
  Utf8,
  // Utf16,
  // Utf32,
}
