//! Ex command attributes.

pub const BANG: &str = "bang";
pub const NARGS: &str = "nargs";

#[derive(
  Debug,
  Copy,
  Clone,
  PartialEq,
  Eq,
  PartialOrd,
  Ord,
  Hash,
  strum_macros::Display,
  strum_macros::EnumString,
)]
pub enum Nargs {
  #[strum(serialize = "0")]
  /// No arguments
  Zero,

  #[strum(serialize = "{0}")]
  /// N arguments
  Count(u8),

  #[strum(serialize = "?")]
  /// 0 or 1 argument
  Optional,

  #[strum(serialize = "+")]
  /// 1 or more arguments
  More,

  #[strum(serialize = "*")]
  /// Any arguments
  Any,
}

#[derive(Debug, Clone)]
pub struct Attributes {
  pub bang: bool,
  pub nargs: Nargs,
}

impl Default for Attributes {
  fn default() -> Self {
    Self {
      bang: false,
      nargs: Nargs::Zero,
    }
  }
}
