//! Ex command runtime context.

use crate::js::binding;
use crate::js::converter::*;
use compact_str::CompactString;

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
  pub bang: bool,

  #[builder(default = ARGS_DEFAULT)]
  pub args: Vec<CompactString>,
}

impl ToV8 for CommandContext {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Value> {
    let obj = v8::Object::new(scope);

    // bang
    let bang_value = to_v8(scope, self.bang);
    binding::set_property_to(scope, obj, BANG, bang_value);

    // args
    let args_value = to_v8(scope, self.args.clone());
    binding::set_property_to(scope, obj, ARGS, args_value);

    obj.into()
  }
}
