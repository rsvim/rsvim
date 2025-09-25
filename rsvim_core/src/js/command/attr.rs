//! Ex command attributes.

use crate::{buf::BufferId, js::build};

/// Command attribute name.
pub const BANG: &str = "bang";
pub const NARGS: &str = "nargs";
pub const BUFFER: &str = "buffer";

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

#[derive(Debug, Copy, Clone, derive_builder::Builder)]
pub struct Attributes {
  #[builder(default = BANG_VALUE)]
  pub bang: bool,

  #[builder(default = NARGS_VALUE)]
  pub nargs: Nargs,

  #[builder(default = BUFFER_VALUE)]
  pub buffer: Option<BufferId>,
}

impl Attributes {
  fn from<'a>(
    scope: &mut v8::HandleScope,
    value: v8::Local<'a, v8::Object>,
  ) -> Self {
    let mut builder = AttributesBuilder::default();

    // bang
    let bang_name = v8::String::new(scope, BANG).unwrap();
    match value.get(scope, bang_name.into()) {
      Some(bang_value) => {
        let bang_value = bang_value.to_boolean(scope).boolean_value(scope);
        builder.bang(bang_value);
      }
      None => { /* do nothing */ }
    }

    builder.build().unwrap()
  }
}
