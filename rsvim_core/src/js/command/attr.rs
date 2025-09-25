//! Ex command attributes.

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

  #[strum(serialize = "?")]
  /// 0 or 1 argument
  Optional,

  #[strum(serialize = "+")]
  /// 1 or more arguments
  AtLeast1,

  #[strum(serialize = "*")]
  /// 0 or 1 argument
  Any,
}
