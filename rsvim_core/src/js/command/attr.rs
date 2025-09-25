//! Ex command attributes.

use crate::state::mode::Modes;

pub const BANG: &str = "bang";
pub const MODS: &str = "mods";
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

  #[strum(serialize = "n")]
  /// N arguments
  Count(u32),

  #[strum(serialize = "?")]
  /// 0 or 1 argument
  Optional,

  #[strum(serialize = "+")]
  /// 1 or more arguments
  OneOrMore,

  #[strum(serialize = "*")]
  /// Any arguments
  Any,
}

#[derive(Debug, Clone)]
pub struct Attributes {
  pub bang: bool,
  pub mods: Modes,
  pub nargs: Nargs,
}
