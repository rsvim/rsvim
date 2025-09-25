//! Ex command attributes.

use crate::buf::BufferId;

/// Command attribute name.
pub const BANG_NAME: &str = "bang";
pub const NARGS_NAME: &str = "nargs";
pub const BUFFER_NAME: &str = "buffer";

/// Default command attributes.
pub const NARGS_VALUE: Nargs = Nargs::Zero;
pub const BANG_VALUE: bool = false;
pub const BUFFER_VALUE: Option<BufferId> = None;
pub const FORCE_VALUE: bool = true;

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
  pub buffer: Option<BufferId>,
}
