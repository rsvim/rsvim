//! Ex command runtime context.

use crate::js::converter::*;
use compact_str::CompactString;
use compact_str::ToCompactString;

/// Command attribute name.
pub const BANG: &str = "bang";
pub const ARGS: &str = "args";

/// Default command attributes.
pub const BANG_DEFAULT: bool = false;
pub const ARGS_DEFAULT: Vec<CompactString> = vec![];

#[derive(Debug, Clone, PartialEq, Eq, derive_builder::Builder)]
pub struct CommandContext {
  #[builder(default = BANG_DEFAULT)]
  // bang
  bang: bool,

  #[builder(default = ARGS_DEFAULT)]
  args: Vec<CompactString>,
}

impl CommandContext {
  pub fn bang(&self) -> bool {
    self.bang
  }

  pub fn args(&self) -> &Vec<CompactString> {
    &self.args
  }
}

impl ToV8 for CommandContext {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Value> {
    let obj = v8::Object::new(scope);

    // bang
    let bang_field = to_v8(scope, BANG);
    let bang_value = to_v8(scope, self.bang());
    obj.set(scope, bang_field, bang_value);

    // args
    let args_field = to_v8(scope, ARGS);
    let args_value = to_v8(scope, self.args().clone());
    obj.set(scope, args_field, args_value);

    obj.into()
  }
}
