//! Ex command attributes.

use crate::js::converter::*;
use std::str::FromStr;

/// Default values.
pub const BANG_DEFAULT: bool = false;
pub const NARGS_DEFAULT: Nargs = Nargs::Zero;

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

  #[strum(serialize = "1")]
  /// 1 argument
  One,

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

impl FromV8 for Nargs {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Self {
    debug_assert!(value.is_string() || value.is_string_object());
    let nargs = value.to_string(scope).unwrap().to_rust_string_lossy(scope);
    Nargs::from_str(&nargs).unwrap()
  }
}

impl ToV8 for Nargs {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Value> {
    self.to_string().to_v8(scope)
  }
}

#[derive(
  Debug,
  Clone,
  PartialEq,
  Eq,
  derive_builder::Builder,
  rsvim_macro::ToV8,
  rsvim_macro::FromV8,
)]
pub struct CommandAttributes {
  #[builder(default = BANG_DEFAULT)]
  pub bang: bool,

  #[builder(default = NARGS_DEFAULT)]
  pub nargs: Nargs,
}
